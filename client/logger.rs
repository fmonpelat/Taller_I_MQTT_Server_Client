struct Logging {
    //funcion para loguear donde se le pasa un string y lo escribe en algún lugar

    
}

//Logger podría ser un mutex para proteger la sección crítica(escritura de archivos)
//Se podría usar un writer que habra un log file y lo escriba alli
impl Logging {
    //implementar el log,open
    // el new es similar a lo que hicimos en la practica con el helado builder
    pub fn new(fileName:str, sourceLog:str) -> Self {
        // chequear si se puede hacer el open para escribir en el archivo, si no existe se crea
        Logging{
            //tener el file abierto y listo para escribir
        }
    }
    pub fn log( message:str) -> Result(){
        //definir estructura del log del mensaje
        //con el archivo abierto escribir en el log
    }
}
