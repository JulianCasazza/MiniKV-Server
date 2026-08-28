use crate::comando::Comando;
use crate::minikv_errors::{ClientError, MinikvError, ServerError};
use std::collections::HashMap;
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::{fs, fs::OpenOptions};

const LOG: &str = ".minikv.log";

const OK: &str = "OK";

pub struct Minikv {
    pub ruta_log: String,
    pub ruta_data: String,
    pub diccionario: HashMap<String, String>,
}

impl Minikv {
    /** Instancia un nuevo Minikv cuyo contenido va a depender de los archivos .log y .data.
    Recibe las rutas de los archivos y los procesa para cargar los datos al Minikv */
    pub fn new(log: String, data: String) -> Result<Self, MinikvError> {
        let mut kv = Minikv {
            ruta_log: log,
            ruta_data: data,
            diccionario: HashMap::new(),
        };

        let Ok(mut data_file) = OpenOptions::new()
            .read(true)
            .create(true)
            .append(true)
            .open(&kv.ruta_data)
        else {
            return Err(ServerError::InvalidDataFile.into());
        };
        kv.obtener_data(&mut data_file)?;
        let _ = data_file.flush();
        let Ok(mut log_file) = OpenOptions::new()
            .read(true)
            .create(true)
            .append(true)
            .open(&kv.ruta_log)
        else {
            return Err(ServerError::InvalidLogFile.into());
        };
        kv.aplicar_log(&mut log_file)?;
        let _ = log_file.flush();

        Ok(kv)
    }
    ///Abre el archivo .data (si existe) y lo lee linea por linea obteniendo los datos e insertandolos en el Minikv
    fn obtener_data(&mut self, data: &mut dyn Read) -> Result<(), MinikvError> {
        let reader = BufReader::new(data);

        for linea_data in reader.lines() {
            let Ok(linea) = linea_data else {
                return Err(ServerError::InvalidDataFile.into());
            };
            let sin_comillas = linea.trim_matches('"');
            if linea == sin_comillas {
                return Err(ServerError::InvalidDataFile.into());
            }

            let par: Vec<&str> = sin_comillas.split("\" \"").collect();
            if let Some(clave) = par.first()
                && let Some(valor) = par.get(1)
            {
                let c = clave.replace("\\\"", "\"");
                self.diccionario.insert(c, valor.replace("\\\"", "\""));
            } else {
                return Err(ServerError::InvalidDataFile.into());
            }
        }
        Ok(())
    }
    ///Abre el archivo .log (si existe) y lo lee linea por linea obteniendo las operaciones de escritura y aplicandolas al Minikv
    fn aplicar_log(&mut self, log: &mut dyn Read) -> Result<(), MinikvError> {
        let reader = BufReader::new(log);

        for linea_log in reader.lines() {
            let comando = parsear_linea_y_construir_comando(linea_log)?;
            if let Some(clave) = comando.clave {
                let c = clave.replace("\\\"", "\"");
                match comando.valor {
                    Some(v) => {
                        self.diccionario.insert(c, v);
                    }
                    None => {
                        self.diccionario.remove(&c);
                    }
                }
            } else {
                return Err(ServerError::InvalidLogFile.into());
            }
        }
        Ok(())
    }

    /// Recibe un parametro de tipo Comando y lo evalua para luego aplicar las operaciones correspondientes según el comando
    pub fn ejecutar_comando(&self, comando: Comando) -> Result<String, MinikvError> {
        match comando.nombre.trim() {
            "get" => {
                if let Some(_v) = comando.valor {
                    Err(ClientError::ExtraArgument.into())
                } else {
                    self.get(comando.clave)
                }
            }
            "snapshot" => {
                if let Some(_c) = comando.clave {
                    Err(ClientError::ExtraArgument.into())
                } else {
                    match self.snapshot() {
                        Ok(_) => Ok(String::from(OK)),
                        Err(_) => Err(ServerError::InvalidDataFile.into()),
                    }
                }
            }
            "length" => {
                if let Some(_c) = comando.clave {
                    Err(ClientError::ExtraArgument.into())
                } else {
                    Ok(self.length())
                }
            }
            _ => Err(ClientError::UnknownCommand.into()),
        }
    }

    /**Si recibe una clave y un valor asocia la clave al valor y lo guarda en el Minikv.
     * Si la clave ya estaba guardada el nuevo valor pisa al anterior.
     * Si recibe un solo argumento (clave) desasocia el valor de la clave**/
    pub fn set(
        &mut self,
        clave: Option<String>,
        valor: Option<String>,
        archivo: &mut dyn Write,
    ) -> Result<String, MinikvError> {
        if let Some(c) = clave {
            match append_log(&c, &valor, archivo) {
                Ok(_) => (),
                Err(_) => return Err(ServerError::InvalidDataFile.into()),
            };
            match valor {
                Some(v) => {
                    self.diccionario.insert(c, v);
                }
                None => {
                    self.diccionario.remove(&c);
                }
            }
        } else {
            return Err(ClientError::MissingArgument.into());
        }
        Ok(OK.to_string())
    }
    ///Busca la clave recibida por parametro en el Minikv y devuelve su valor asociado
    fn get(&self, clave: Option<String>) -> Result<String, MinikvError> {
        if let Some(c) = clave {
            match self.diccionario.get(&c) {
                Some(valor) => Ok(valor.replace("\\\"", "\"")),
                None => Err(ClientError::NotFound.into()),
            }
        } else {
            Err(ClientError::MissingArgument.into())
        }
    }
    ///Trunca el archivo .log y escribe en el archivo .data todos los pares calve-valor del Minikv
    fn snapshot(&self) -> Result<(), std::io::Error> {
        fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(LOG)?;

        let archivo_data = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.ruta_data)?;

        let mut writer = BufWriter::new(archivo_data);

        for (clave, valor) in &self.diccionario {
            writeln!(
                writer,
                "\"{}\" \"{}\"",
                clave.replace('"', "\\\""),
                valor.replace('"', "\\\"")
            )?;
        }
        writer.flush()?;

        Ok(())
    }
    ///Devuelve la cantidad de elementos guardados en el Minikv
    fn length(&self) -> String {
        format!("{}", self.diccionario.len())
    }

    #[cfg(test)]
    pub fn new_testing() -> Self {
        Minikv {
            ruta_log: "".to_string(),
            ruta_data: "".to_string(),
            diccionario: HashMap::new(),
        }
    }
}
/// Agrega al final del archivo .log la operación de escritura que se haya realizado en la ejecució, si se realizó alguna
fn append_log(
    clave: &str,
    valor: &Option<String>,
    archivo: &mut dyn Write,
) -> Result<(), std::io::Error> {
    let mut writer = BufWriter::new(archivo);
    match valor {
        Some(v) => {
            writeln!(
                writer,
                "set \"{}\" \"{}\"",
                clave.replace('"', "\\\""),
                v.replace('"', "\\\"")
            )?;
        }
        None => {
            writeln!(writer, "set \"{}\"", clave.replace('"', "\\\""))?;
        }
    }
    writer.flush()?;

    Ok(())
}

fn parsear_linea_y_construir_comando(linea: io::Result<String>) -> Result<Comando, MinikvError> {
    let Ok(linea) = linea else {
        return Err(ServerError::InvalidLogFile.into());
    };
    let Some(cont) = linea.strip_prefix("set ") else {
        return Err(ServerError::InvalidLogFile.into());
    };
    let sin_comillas = cont.trim_matches('"').to_string();

    let par: Vec<&str> = sin_comillas.split("\" \"").collect();

    let mut args: Vec<String> = vec!["set ".to_string()];
    for p in par {
        args.push(p.to_string());
    }
    Ok(Comando::parsear_comando(&args))
}

#[cfg(test)]
mod tests {
    use crate::minikv::Comando;
    use crate::minikv::Minikv;
    use std::io::Cursor;
    use std::{
        fs::File,
        io::{BufRead, BufReader},
    };

    #[test]
    fn set_y_get_test() {
        let mut log = Cursor::new(Vec::new());
        let mut mini_kv = Minikv::new_testing();
        mini_kv
            .set(
                Some("clave 1".to_string()),
                Some("valor 1".to_string()),
                &mut log,
            )
            .unwrap();
        assert_eq!(
            mini_kv.get(Some("clave 1".to_string())).unwrap(),
            "valor 1".to_string()
        );
        mini_kv
            .set(
                Some("\"hola\"".to_string()),
                Some("\"mundo\"".to_string()),
                &mut log,
            )
            .unwrap();
        assert_eq!(
            mini_kv.get(Some("\"hola\"".to_string())).unwrap(),
            "\"mundo\"".to_string()
        );
    }

    #[test]
    fn length_test() {
        let mut log = Cursor::new(Vec::new());
        let mut mini_kv = Minikv::new_testing();
        assert_eq!(mini_kv.length(), "0".to_string());
        mini_kv
            .set(
                Some("clave 1".to_string()),
                Some("valor 1".to_string()),
                &mut log,
            )
            .unwrap();
        assert_eq!(mini_kv.length(), "1".to_string());
        mini_kv
            .set(
                Some("clave 1".to_string()),
                Some("valor A".to_string()),
                &mut log,
            )
            .unwrap();
        assert_eq!(mini_kv.length(), "1".to_string());
    }

    #[test]
    fn obtener_data_test() {
        let contenido = "\"clave 1\" \"valor 1\"\n\"Rayuela\" \"Cortazar\"";
        let mut data = Cursor::new(contenido);
        let mut mini_kv = Minikv::new_testing();
        let _ = mini_kv.obtener_data(&mut data);
        assert_eq!(mini_kv.get(Some("clave 1".to_string())).unwrap(), "valor 1");
        assert_eq!(
            mini_kv.get(Some("Rayuela".to_string())).unwrap(),
            "Cortazar"
        );
    }

    #[test]
    fn aplicar_log_test() {
        let contenido = "set \"Ficciones\" \"Borges\"\nset \"Martin Fierro\" \"Hernandez\"";
        let mut log = Cursor::new(contenido);
        let mut mini_kv = Minikv::new_testing();
        let _ = mini_kv.aplicar_log(&mut log);
        println!("{:?}", mini_kv.diccionario);
        assert_eq!(
            mini_kv.get(Some("Ficciones".to_string())).unwrap(),
            "Borges"
        );
        assert_eq!(
            mini_kv.get(Some("Martin Fierro".to_string())).unwrap(),
            "Hernandez"
        );
    }
    #[test]
    fn snapshot_test() {
        let kv = Minikv::new(
            "tests/.snapshot_test.log".to_string(),
            "tests/.snapshot_test.data".to_string(),
        )
        .unwrap();
        let comando_args = vec![
            "set".to_string(),
            "\"clave 1\"".to_string(),
            "\"valor 1\"".to_string(),
        ];
        let comando = Comando::parsear_comando(&comando_args);
        let _ = kv.ejecutar_comando(comando);
        kv.snapshot().unwrap();
        let data = File::open(kv.ruta_data).unwrap();
        let reader = BufReader::new(data);
        for linea in reader.lines() {
            assert_eq!(linea.unwrap(), "\"\\\"clave 1\\\"\" \"\\\"valor 1\\\"\"");
        }
    }
}
