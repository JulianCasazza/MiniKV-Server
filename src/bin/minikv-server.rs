use minikv_server::comando::{self, Comando};
use minikv_server::minikv::{self, Minikv};
use minikv_server::minikv_errors::{ClientError, CommunicationError, MinikvError, ServerError};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock, mpsc};
use std::{
    env,
    net::TcpListener,
    thread::{self, JoinHandle},
};

const LOG: &str = ".minikv.log";
const DATA: &str = ".minikv.data";

fn main() {
    let args: Vec<String> = env::args().collect();
    let Some(direccion) = args.get(1) else {
        eprintln!("ERROR \"{}\"", ServerError::ServerSocketBinding);
        return;
    };
    let mini_kv = match minikv::Minikv::new(String::from(LOG), String::from(DATA)) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("ERROR \"{}\"", e);
            return;
        }
    };
    let minikv_compartido = Arc::new(RwLock::new(mini_kv));
    let Ok(listener) = TcpListener::bind(direccion) else {
        eprintln!("ERROR: \"{}\"", ServerError::ServerSocketBinding);
        return;
    };
    let mut handles: Vec<JoinHandle<()>> = vec![];
    server_run(listener, &mut handles, minikv_compartido);
    for handle in handles {
        if handle.join().is_ok() {}
    }
}
///Por cada Tcp Stream que se conecta crea un nuevo thread en el que se van a manejar las operaciones de cada cliente
fn server_run(
    listener: TcpListener,
    handles: &mut Vec<JoinHandle<()>>,
    mini_kv_arc: Arc<RwLock<Minikv>>,
) {
    let (tx, rx) = mpsc::channel::<ServerError>();
    for stream in listener.incoming() {
        let minikv_clone = Arc::clone(&mini_kv_arc);
        let sender_clone = tx.clone();
        let handle: JoinHandle<()> = thread::spawn(move || {
            let Ok(s) = stream else {
                eprintln!("{}", CommunicationError::ConnectionClosed);
                return;
            };

            let Ok(s_writer) = s.try_clone() else {
                return;
            };
            let reader = BufReader::new(s);
            interpretar_lineas(reader, s_writer, minikv_clone, sender_clone);
        });
        if let Ok(_server_error) = rx.try_recv() {
            return;
        }
        handles.push(handle);
    }
}

fn parsear_linea(line: String) -> Result<Vec<String>, MinikvError> {
    let mut args: Vec<String> = vec![];
    let mut actual = String::new();
    let mut entre_comillas = false;
    let mut escapado = false;
    for c in line.trim().chars() {
        if escapado {
            actual.push(c);
            escapado = false;
        } else if c == '\\' {
            escapado = true;
        } else if c == '"' {
            entre_comillas = !entre_comillas;
        } else if c == ' ' && !entre_comillas {
            if !actual.trim().is_empty() {
                args.push(actual);
                actual = String::new();
            }
        } else {
            actual.push(c);
        }
    }
    if !actual.is_empty() {
        args.push(actual);
    }
    if args.len() > 3 {
        return Err(ClientError::ExtraArgument.into());
    }
    Ok(args)
}
///Dependiendo del comando recibido se evalua si es necesario un lock de escritura o si se usa el de lectura
fn evaluar_comando(comando: Comando, minikv: Arc<RwLock<Minikv>>) -> Result<String, MinikvError> {
    match comando.nombre.trim() {
        "set" => {
            let mini_kv_write = minikv.write();
            if let Ok(mut kv) = mini_kv_write {
                let Ok(mut archivo) = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&kv.ruta_log)
                else {
                    return Err(ServerError::InvalidLogFile.into());
                };
                kv.set(comando.clave, comando.valor, &mut archivo)
            } else {
                Err(ServerError::InvalidArgs.into())
            }
        }
        _ => {
            let mini_kv_read = minikv.read();
            match mini_kv_read {
                Ok(kv) => kv.ejecutar_comando(comando),
                Err(_) => Err(ServerError::InvalidArgs.into()),
            }
        }
    }
}

fn interpretar_lineas(
    reader: BufReader<TcpStream>,
    mut s_writer: TcpStream,
    minikv_clone: Arc<RwLock<Minikv>>,
    sender: Sender<ServerError>,
) {
    let mut lines = reader.lines();
    while let Some(Ok(line)) = lines.next() {
        let linea = match parsear_linea(line) {
            Ok(l) => l,
            Err(e) => {
                let continue_running = evaluar_error(e, &mut s_writer, sender.clone());
                if !continue_running {
                    return;
                }
                continue;
            }
        };
        let comando = comando::Comando::parsear_comando(&linea);
        match evaluar_comando(comando, Arc::clone(&minikv_clone)) {
            Ok(result) => {
                if s_writer.write(result.as_bytes()).is_err()
                    || s_writer.write("\n".as_bytes()).is_err()
                {
                    return;
                }
            }
            Err(e) => {
                let continue_running = evaluar_error(e, &mut s_writer, sender.clone());
                if !continue_running {
                    return;
                }
            }
        }
    }
}
///Dependiendo el tipo de error se decide como continuará el comportamiento de la conexión
fn evaluar_error(
    error: MinikvError,
    s_writer: &mut TcpStream,
    sender_clone: Sender<ServerError>,
) -> bool {
    match error {
        MinikvError::Client(error) => {
            let mensaje = format!("ERROR \"{}\"\n", error);
            let _response = s_writer.write_all(mensaje.as_bytes());
            true
        }
        MinikvError::Communication(error) => {
            let mensaje = format!("ERROR \"{}\"\n", error);
            let _response = s_writer.write_all(mensaje.as_bytes());
            eprintln!("ERROR \"{}\"", mensaje);
            true
        }
        MinikvError::Server(error) => {
            eprintln!("ERROR \"{}\"", error);
            let _error_send = sender_clone.send(error);
            false
        }
    }
}
