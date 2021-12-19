# MQTT Rústico - Taller de Programación 1


## Equipo - Los Rustmonnaz

Tabla de contenidos
=================

<!--ts-->
   * [Integrantes](#integrantes)
   * [Objetivo del trabajo práctico](#objetivo-del-trabajo-práctico)
   * [Comandos de ejecución](#comandos-de-ejecución)
      * [Iniciando el servidor](#iniciando-el-servidor)
      * [Iniciando el cliente](#iniciando-el-cliente)

   * [Iniciando la conexión con el servidor](#iniciando-la-conexión-con-el-servidor)
      * [Connection](#connection)
      * [Publish](#publish)
      * [Subscriptions](#subscriptions)
      * [Messages](#messages)

   * [Conexión del servidor o Broker](#conexión-del-servidor-o-broker)
<!--te-->

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
  
   Para ejecutar el servidor, se debe ir al directorio donde se encuentra el mismo y ejecutar en línea de comandos lo siguiente:
   ```sh
        cargo run server
    ```
    Los parámetros con los que se inicia el servidor se encuentran especificados en el archivo de configuración llamado _config.yaml_ que contiene tanto: 
    * **host:** Dirección del servidor.
    * **port:** Puerto en cual el servidor escuchará por solicitudes
    * **logfile:** Path del archivo donde se irán almacenando todos los registros tanto de las solicitudes como de las acciones que se van realizando.
    * **credentials_file:** Path del archivo el cual servidor levanta los datos de los usuarios que pueden conectarse en forma segura.

* **Iniciando el cliente**

    Para ejecutar el cliente se debe ir al directorio del mismo y ejecutar en línea de comandos lo siguiente:
    ```sh
        cargo run cliente
    ```

    Una vez ejecutado correctamente el cliente, se abrirá la interfaz para que el cliente pueda conectarse al servidor.
    
    Una vista de la ventana principal para la conexón del cliente es la siguiente:
    ![main_window](/images/main_window.png "Ventana principal")

    Los parámetros con los que se debe iniciar el cliente son los siguientes:
    * **Server host:** Dirección del servidor.
    * **Server Port:** Puerto del servidor.
    * **Clean session:** Puede estar activada o desactivada. Para cuando se realiza la conexión con el Clean session activado, si el subscriber pierde la conexión, entonces todos los mensajes que hayan sido publicados a ese topic no van a ser almacenados y cuando el subscriber se reconecta, habrá perdido estos mensajes. Caso contrario, si el Clean session ha sido desactivado, entonces los mensajes serán almacenados y entregados al cliente cuando este se reconecte.
    * **Connect secure:** Servicio al cual los clientes se pueden autenticar o no al conectarse al servidor. Los clientes al autenticarse deben ingresar tanto el usuario como la contraseña.


* **Iniciando la conexión con el servidor**

    Una vez conectado con el servidor el cliente podrá tanto publicar o suscribirse a los tópicos que desee.

    Un ejemplo de una conexión mediante MQTT es el siguiente:

    ![connection_window](/images/connection_window.png "Ventana de conexión")

    Las secciones de cada parte de la ventana de conexión se explican a continuación:

    **- Connection:** 

    Especifica los datos del cliente que pudo conectarse al servidor.
    En esta sección también se encuentra el botón Disconnect para que el cliente pueda desconectarse del servidor.

    **- Publish**

    Una vez que un cliente MQTT está conectado, puede publicar mensajes. En esta sección se puede ingresar un mensaje, un tópico, indicar el nivel de calidad de servicio (Quality of Service, QoS) para la entrega de datos y también indicar si desea Retain message por si los clientes que se suscriban a ese tema recibirán el último mensaje retenido sobre ese tema inmediatamente después de suscribirse.

    Cuando el broker confirma la recepción del mensaje, se señaliza o no el éxito de la operación en la sección Messages mediante el mensaje **PUBACK** dependiendo el QoS utilizado.

    A continuación se explica en profundidad cada parámetro:

    * **Topic:** Es un simple texto que el usuario puede ingresar.

            Nota: Debido a un recorte del scope del trabajo práctico, los Wilcards no fueron implementados.
    
    * **QoS:** o Calidad de Servicio que afecta directamente la conexión y la seguridad que se brinda para el envío de mensajes. Existen tres tipos de QoS pero por consideraciones del trabajo práctico, los que se pueden configurar son los siguientes:
        1. **QoS 0:** «At most once«, esta QoS asegura que se entregará el mensaje «a lo sumo, una vez», por lo que existe la posibilidad de que el mensaje no sea entregado. Es la QoS más ligera para la red y la que menor seguridad brinda.
        2. **QoS 1:** «At least once«, esta QoS asegura que el mensaje será entregado «como mínimo, una vez», esto quiere decir que pueden producirse duplicados del mensaje. Cuando el mensaje se publica, entonces el cliente espera el PUBACK (acuse de recibido) desde el broker.
    
    * **Retain:** brinda la posibilidad de almacenar el último mensaje publicado a un topic específico y cuando un nuevo subscriber se conecte, este reciba el último mensaje enviado a ese topic.

    * **Message:** Es el texto que se desea publicar al tópico correspondiente.

    **- Subscriptions**

    Sección en la cual un cliente se suscribe a uno o varios _topics_ para recibir los mensajes que han sido publicados en ellos.

    Cuando el broker confirma la recepción del mensaje, se señaliza éxito de la operación en la sección Messages mediante el mensaje **SUBACK**.


    **- Messages**

    En esta sección el cliente puede observar en tiempo real todos los mensajes que recibe del broker, tanto de sus publicaciones como de los tópicos al cual se suscribió.


* **Conexión del servidor o Broker**

    Es el corazón del protocolo MQTT. Principalmente el encargado de gestionar el flujo de mensajes; recibe todos los mensajes, los filtra, decide quién está interesado en él y luego envia el mensaje a todos los clientes suscritos.

    Además, el broker puede manejar hasta varios clientes MQTT conectados simultáneamente.

    También contiene la sesión de todos los clientes persistentes, incluidas las suscripciones y los mensajes retenidos. Otra responsabilidad del broker es la autenticación y autorización de los clientes que requieren conectarse de forma segura. Como se mencionó más arriba, el servidor levanta de un archivo las credenciales de los clientes que pueden autenticarse al mismo; en él se tiene los usuarios y contraseñas de los clientes.

    Adicionalmente, el broker almacena todos los registros tanto de las solicitudes como de las acciones que se van realizando en un archivo log para su posterior análisis de ser necesario.
