///Estructura que agrupa el contenido que puede tener un comando válido
pub struct Comando {
    pub nombre: String,
    pub clave: Option<String>,
    pub valor: Option<String>,
}

impl Comando {
    ///Instancia un nuevo comando solo con un nombre
    pub fn new(nombre_comando: String) -> Self {
        Comando {
            nombre: nombre_comando,
            clave: None,
            valor: None,
        }
    }
    ///Crea un nuevo Comando a partir de los datos de un array
    pub fn parsear_comando(args: &[String]) -> Comando {
        let keyword = match args.first() {
            Some(s) => s,
            None => "",
        };
        let mut comando: Comando = Comando::new(keyword.to_string());
        if let Some(nombre_clave) = args.get(1) {
            comando.clave = Some(nombre_clave.to_string())
        };
        if let Some(nombre_valor) = args.get(2) {
            comando.valor = Some(nombre_valor.to_string())
        };
        comando
    }
}

#[cfg(test)]
mod tests {
    use crate::comando::Comando;
    #[test]
    fn comando_1_argumento() {
        let args: Vec<String> = vec!["length".to_string()];
        let comando = Comando::parsear_comando(&args);
        //assert!(comando.nombre == "length");
        assert!(comando.clave.is_none());
        assert!(comando.valor.is_none());
    }
    #[test]
    fn comando_2_argumentos() {
        let args: Vec<String> = vec!["get".to_string(), "clave 1".to_string()];
        let comando = Comando::parsear_comando(&args);
        //assert!(comando.nombre == "get");
        assert!(comando.clave == Some(String::from("clave 1")));
        assert!(comando.valor.is_none());
    }
    #[test]
    fn comando_3_argumentos() {
        let args: Vec<String> = vec![
            "set".to_string(),
            "clave 2".to_string(),
            "valor 2".to_string(),
        ];
        let comando = Comando::parsear_comando(&args);
        //assert!(comando.nombre == "set");
        assert!(comando.clave == Some(String::from("clave 2")));
        assert!(comando.valor == Some(String::from("valor 2")));
    }
}
