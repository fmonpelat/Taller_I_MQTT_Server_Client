# MQTT Rústico - Taller de Programación 1


## Equipo:  Los Rustmonnaz

Tabla de contenidos
=================

<!--ts-->
   * [Integrantes](#integrantes)
   * [Objetivo del trabajo práctico](#objetivo-del-trabajo-práctico)
   * [Programas](#programas)
      * [Iniciando el servidor](#iniciando-el-servidor)
      * [Iniciando el cliente](#iniciando-el-cliente)
   * [Conexión con el servidor](#conexión-con-el-servidor)
      * [Connection](#connection)
      * [Publish](#publish)
      * [Subscriptions](#subscriptions)
      * [Messages](#messages)
   * [Conexión del servidor](#conexión-del-servidor)
<!--te-->

Integrantes
===========
* Facundo Monpelat
* Cynthia Gamarra

Objetivo del trabajo práctico
=============================

Se requirió la implementación del protocolo MQTT el cual es un protocolo de mensajería basado en el patrón de comunicación _publisher-suscriber_ y la arquitectura _cliente-servidor_. 

Para este trabajo práctico se utilizó la versión 3.1.1 del protocolo. La especificación del mismo puede encontrarse en el siguiente link: [ MQTT-v3.1.1](http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.pdf).

La especificación de los requerimientos del trabajo práctico se encuentra en el siguiente link: [ Enunciado](https://taller-1-fiuba-rust.github.io/proyecto_2C2021.html).

**Nota:** Dado la cantidad integrantes en el equipo no se implementó la totalidad de los requerimientos del trabajo práctico.

Programas
======================

Iniciando el servidor
---------------------
  
Para ejecutar el servidor, se debe ir al directorio donde se encuentra el mismo (`./server`) y ejecutar en línea de comandos lo siguiente:
```sh
   cargo run server
```
Los parámetros con los que se inicia el servidor se encuentran especificados en el archivo de configuración llamado _config.yaml_ que contiene tanto: 
* **host:** Dirección del servidor.
* **port:** Puerto en cual el servidor escuchará por solicitudes
* **logfile:** Path del archivo donde se irán almacenando todos los registros tanto de las solicitudes como de las acciones que se van realizando.
* **credentials_file:** Path del archivo el cual el servidor carga los datos de los usuarios que pueden conectarse de forma segura. 

### Credential file
El archivo Credential file contiene los usuarios y contraseñas en forma de clave valor por ejemplo:
```
user1: contraseña
```
esta habilitados los comentarios para deshabilitar un usuario del archivo sin borralo usando el caracter `'#'`, por ejemplo si queremos deshabilitar al `user1` o `user2` de conectarse con el server:
```
# user1: contraseña
#user2: contraseña
```

_________________

Iniciando el cliente CLI
--------------------
Para ejecutar el cliente en modo CLI (linea por consola) ir al directorio (`./client`) y y ejecutar en línea de comandos lo siguiente:
```sh
  cargo run client
```
Una vez ejecutado correctamente el cliente, se abrirá la interfaz para que el cliente pueda conectarse al servidor.

### Comandos
Para interactuar con el cliente en modo cli, los comandos se deben ingresar por stdin.
Los comandos habilitados son: 

**Nota:** En mayuscula se especifican los parametros a modificar, todos los comandos se escriben en minusculas:
#### **<em>connect HOST PORT USER PASSWORD</em>**
Este comando conecta al cliente con el servidor, es opcional el uso de los placeholder `USER` y `PASSWORD`.
#### **<em>publish DUP QOS RETAIN TOPIC MESSAGE </em>** 
Para enviar un mensaje con topic y el mensaje. Valores validos de dup, qos y retain: 0, 1.

#### **<em>subscribe QOS TOPIC</em>**
Para subscribirse a un topico, valores validos de QOS: 0, 1.

### **<em>disconnect</em>**
Para desconectarse del servidor de una manera grata.

_________________

Iniciando el GUI
--------------------

Para ejecutar el cliente se debe ir al directorio del mismo y ejecutar en línea de comandos lo siguiente:
```sh
  cargo run gui
```

Una vez ejecutado correctamente la gui, se abrirá la interfaz para que el cliente pueda conectarse al servidor.
    
Una vista de la ventana principal para la conexón del cliente es la siguiente:
![main_window](/images/main_window.png "Ventana principal")

Los parámetros con los que se debe iniciar el cliente son los siguientes:
* **Server host:** Dirección del servidor.
* **Server Port:** Puerto del servidor.
* **Clean session:** Puede estar activada o desactivada. Para cuando se realiza la conexión con la opcion `Clean session` activado, se inicia una sesion nueva.

* **Last will testament:** Utilidad que proporciona una forma para que los clientes respondan a desconexiones _ungracefully_ de una manera adecuada. Se se utiliza para notificar a los suscriptores de un cierre inesperado del _publisher_.
   
   El proceso básico es el siguiente:

   1. El _publisher_ le dice al broker que notifique a todos los suscriptores a un tema, utilizando el último mensaje de voluntad( **Will topic** ), en caso de que se rompa la conexión.
   2. Si el broker detecta una interrupción de la conexión, envía el mensaje de última voluntad a todos los suscriptores de ese tema( **Will message** ).


* **Connect secure:** Servicio al cual los clientes se pueden autenticar o no al conectarse al servidor. Los clientes al autenticarse deben ingresar tanto el usuario como la contraseña.


Conexión con el servidor
-------------------------

Una vez conectado con el servidor el cliente podrá tanto publicar o suscribirse a los tópicos que desee.

Un ejemplo de una conexión mediante MQTT es el siguiente:

![connection_window](/images/connection_window.png "Ventana de conexión")

Las secciones de cada parte de la ventana de conexión se explican a continuación:

Connection
-----------

Especifica los datos del cliente que pudo conectarse al servidor.
En esta sección también se encuentra el botón Disconnect para que el cliente pueda desconectarse del servidor.

Publish
--------
Una vez que un cliente MQTT está conectado, puede publicar mensajes. En esta sección se puede ingresar un mensaje, un tópico, indicar el nivel de calidad de servicio (Quality of Service, QoS) para la entrega de datos y también indicar si desea Retain message por si los clientes que se suscriban a ese tema recibirán el último mensaje retenido sobre ese tema inmediatamente después de suscribirse.


Cuando el broker confirma la recepción del mensaje, se señaliza o no el éxito de la operación en la sección Messages mediante el mensaje **PUBACK** dependiendo el QoS utilizado.


A continuación se explica en profundidad cada parámetro:


* **Topic:** Es un simple texto que el usuario puede ingresar.

            Nota: Debido a un recorte del scope del trabajo práctico, los Wilcards no fueron implementados. 

* **QoS:** o Calidad de Servicio que afecta directamente la conexión y la seguridad que se brinda para el envío de mensajes. Existen tres tipos de QoS pero por consideraciones del trabajo práctico, los que se pueden configurar son los siguientes:

  - **QoS 0:** «At most once«, esta QoS asegura que se entregará el mensaje «a lo sumo, una vez», por lo que existe la posibilidad de que el mensaje no sea entregado. Es la QoS más ligera para la red y la que menor seguridad brinda.

  - **QoS 1:** «At least once«, esta QoS asegura que el mensaje será entregado «como mínimo, una vez», esto quiere decir que pueden producirse duplicados del mensaje. Cuando el mensaje se publica, entonces el cliente espera el PUBACK (acuse de recibido) desde el broker.
    
* **Retain:** brinda la posibilidad de almacenar el último mensaje publicado a un topic específico y cuando un nuevo subscriber se conecte, este reciba el último mensaje enviado a ese topic.

* **Message:** Es el texto que se desea publicar al tópico correspondiente.

Subscriptions
-------------

Sección en la cual un cliente se suscribe a uno o varios _topics_ para recibir los mensajes que han sido publicados en ellos.

Cuando el broker confirma la recepción del mensaje, se señaliza éxito de la operación en la sección Messages mediante el mensaje **SUBACK**.


Messages
--------

En esta sección el cliente puede observar en tiempo real todos los mensajes que recibe del broker, tanto de sus publicaciones como de los tópicos al cual se suscribió.

Conexión del servidor
---------------------

Es el corazón del protocolo MQTT. Principalmente el encargado de gestionar el flujo de mensajes; recibe todos los mensajes, los filtra, decide quién está interesado en él y luego envia el mensaje a todos los clientes suscritos.

Además, el broker puede manejar hasta varios clientes MQTT conectados simultáneamente.

También contiene la sesión de todos los clientes persistentes, incluidas las suscripciones y los mensajes retenidos. Otra responsabilidad del broker es la autenticación y autorización de los clientes que requieren conectarse de forma segura. Como se mencionó más arriba, el servidor levanta de un archivo las credenciales de los clientes que pueden autenticarse al mismo; en él se tiene los usuarios y contraseñas de los clientes.

Adicionalmente, el broker almacena todos los registros tanto de las solicitudes como de las acciones que se van realizando en un archivo log para su posterior análisis de ser necesario.

Un ejemplo del funcionamiento del broker es el siguiente:

![server_process](/images/server_process.png "Proceso del broker")
