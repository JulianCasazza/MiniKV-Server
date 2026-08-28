# MiniKV Server 🦀

> **Contexto Académico:** Este proyecto fue desarrollado como Trabajo Práctico individual para la materia [Taller de Programación I - Cátedra Deymonnaz](https://taller-1-fiuba-rust.github.io/inicio.html) de la [Facultad de Ingeniería de la Universidad de Buenos Aires (FIUBA)](http://www.fi.uba.ar/).

MiniKV Server es un sistema de almacenamiento clave-valor persistente escrito íntegramente en Rust. Permite a múltiples clientes operar de forma concurrente sobre la misma base de datos en memoria, garantizando la persistencia mediante *append-only logs* y *snapshots*.

## 🚀 Características Técnicas

- **Cero dependencias externas:** Implementado utilizando exclusivamente la biblioteca estándar (`std`) de Rust.
- **Concurrencia segura:** Arquitectura cliente-servidor multihilo. Los recursos compartidos se gestionan mediante el patrón de exclusión mutua con `Arc<RwLock>`, permitiendo múltiples lecturas simultáneas de forma eficiente.
- **Protocolo TCP:** Comunicación en red a través de sockets TCP, con manejo estricto de buffers y errores.
- **Persistencia:** Recuperación de estado ante caídas mediante un archivo de transacciones (`.log`) y compactación de datos (`.data`).

## 🛠️ Estructura y Ejecución

El proyecto está compuesto por dos binarios. Para probarlo localmente, abrí dos terminales distintas:

### 1. Iniciar el Servidor
El servidor recibe como argumento la dirección y el puerto donde escuchará las conexiones entrantes.

```bash
cargo run --bin minikv-server -- 127.0.0.1:8080
```

### 2. Conectar un Cliente

El cliente recibe la dirección del servidor y lee los comandos desde la entrada estándar (`STDIN`).
```bash
cargo run --bin minikv-client -- 127.0.0.1:8080
```
## 💻 Interfaz y Comandos

Una vez que el cliente está corriendo, podés ingresar los siguientes comandos por consola (presionando `Enter` para enviar cada operación):

-   **Asociar un valor (`set`):**
    ```
    set "clave" "valor"
    ```
    
-   **Obtener un valor (`get`):**
    ```
    get "clave"
    ```
    
-   **Eliminar un valor:**
    ```
    set "clave"
    ```
    
-   **Cantidad de claves almacenadas (`length`):**
    ```
    length
    ```
    
-   **Compactar la base de datos (`snapshot`):**
    ```
    snapshot
    ```
    
    _(Genera una copia del estado actual y trunca el log de transacciones para ahorrar espacio)._
    
## 💾 Persistencia de Datos

El sistema genera y utiliza automáticamente dos archivos en el directorio donde se ejecuta el servidor:

-   `.minikv.log`: Registro _append-only_ de todas las operaciones de escritura (sets/deletes).
    
-   `.minikv.data`: Último _snapshot_ consolidado con el estado completo de la base de datos.
