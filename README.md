# MQTT Rústico - Taller de Programación 1


## Equipo - Los Rustmonnaz
### Integrantes
* Facundo Monpelat
* Cynthia Gamarra

### Objetivo del trabajo práctico
Se requirió la implementación del protocolo MQTT el cual es un protocolo de mensajería basado en el patrón de comunicación _publisher-suscriber_ y la arquitectura _cliente-servidor_. 

Para este trabajo práctico se utilizó la versión 3.1.1 del protocolo. La especificación del mismo puede encontrarse en el siguiente link: [ MQTT-v3.1.1](http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.pdf).

La especificación de los requerimientos del trabajo práctico se encuentra en el siguiente link: [ Enunciado](https://taller-1-fiuba-rust.github.io/proyecto_2C2021.html).

**Nota:** Dado la cantidad integrantes en el equipo no se implementó la totalidad de los requerimientos del trabajo práctico.

### Comandos de ejecución
* **Iniciando el servidor**
  
   Para poder ejecutar el servidor, se debe ir al directorio donde se encuentra el mismo y ejecutar en línea de comandos lo siguiente:
   ```sh
        cargo run server
    ```
    Los parámetros con los que se inicia el servidor se encuentran especificados en el archivo de configuración llamado _config.yaml_