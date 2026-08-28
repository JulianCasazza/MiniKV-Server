use std::fmt;

#[derive(Debug)]
///Enum que contiene los errores de servidor
pub enum ServerError {
    InvalidArgs,
    ServerSocketBinding,
    InvalidDataFile,
    InvalidLogFile,
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mensaje_error = match self {
            ServerError::InvalidArgs => "INVALID ARGS",
            ServerError::ServerSocketBinding => "SERVER SOCKET BINDING",
            ServerError::InvalidDataFile => "INVALID DATA FILE",
            ServerError::InvalidLogFile => "INVALID LOG FILE",
        };
        write!(f, "{}", mensaje_error)
    }
}

#[derive(Debug)]
///Enum que contiene los errores de cliente
pub enum ClientError {
    NotFound,
    ExtraArgument,
    MissingArgument,
    UnknownCommand,
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mensaje_error = match self {
            ClientError::NotFound => "NOT FOUND",
            ClientError::ExtraArgument => "EXTRA ARGUMENT",
            ClientError::MissingArgument => "MISSING ARGUMENT",
            ClientError::UnknownCommand => "UNKNOWN COMMAND",
        };
        write!(f, "{}", mensaje_error)
    }
}

#[derive(Debug)]
///Enum que contiene los errores de comunicacion
pub enum CommunicationError {
    TimeOut,
    ConnectionClosed,
    ClientSocketBinding,
}

impl fmt::Display for CommunicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mensaje_error = match self {
            CommunicationError::TimeOut => "TIMEOUT",
            CommunicationError::ConnectionClosed => "CONNECTION CLOSED",
            CommunicationError::ClientSocketBinding => "CLIENT SOCKET BINDING",
        };
        write!(f, "{}", mensaje_error)
    }
}

#[derive(Debug)]
///Enum que agrupa los 3 distintos tipos de error que se pueden encontrar en el sitema
pub enum MinikvError {
    Server(ServerError),
    Client(ClientError),
    Communication(CommunicationError),
}

impl From<ServerError> for MinikvError {
    fn from(e: ServerError) -> MinikvError {
        MinikvError::Server(e)
    }
}

impl From<ClientError> for MinikvError {
    fn from(e: ClientError) -> MinikvError {
        MinikvError::Client(e)
    }
}

impl From<CommunicationError> for MinikvError {
    fn from(e: CommunicationError) -> MinikvError {
        MinikvError::Communication(e)
    }
}

impl fmt::Display for MinikvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MinikvError::Server(server_e) => write!(f, "{}", server_e),
            MinikvError::Client(client_e) => write!(f, "{}", client_e),
            MinikvError::Communication(communication_e) => write!(f, "{}", communication_e),
        }
    }
}
