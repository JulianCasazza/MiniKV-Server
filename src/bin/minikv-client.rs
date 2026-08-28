use minikv_server::minikv_errors::{CommunicationError, MinikvError, ServerError};
use std::io::Write;
use std::{
    env,
    io::{BufRead, BufReader, Read, stdin},
    net::TcpStream,
    time::Duration,
};

const TIME_OUT: u64 = 3;

fn main() {
    let args: Vec<String> = env::args().collect();
    let Some(direccion) = args.get(1) else {
        eprintln!("ERROR \"{}\"", CommunicationError::ClientSocketBinding);
        return;
    };

    match client_run(direccion, &mut stdin()) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("ERROR \"{}\"", e);
        }
    }
}

fn client_run(direccion: &String, stdin: &mut dyn Read) -> Result<(), MinikvError> {
    let Ok(socket) = TcpStream::connect(direccion) else {
        return Err(CommunicationError::ClientSocketBinding.into());
    };

    let reader = BufReader::new(stdin);
    let Ok(mut socket_writer) = socket.try_clone() else {
        return Err(CommunicationError::ClientSocketBinding.into());
    };
    socket
        .set_read_timeout(Some(Duration::new(TIME_OUT, 0)))
        .map_err(|_| CommunicationError::TimeOut)?;
    let mut response_reader = BufReader::new(socket);
    for line in reader.lines().map_while(Result::ok) {
        let contenido = socket_writer.write_all(line.as_bytes());
        let salto_de_linea = socket_writer.write_all("\n".as_bytes());
        if contenido.is_err() || salto_de_linea.is_err() {
            return Err(ServerError::InvalidArgs.into());
        }
        let mut response = String::new();
        if response_reader.read_line(&mut response).is_ok() {
            print!("{}", response);
        } else {
            return Err(CommunicationError::ConnectionClosed.into());
        }
    }
    Ok(())
}
