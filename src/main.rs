/* 
============================================================================
   FASE 1 — ANÁLISIS DE SEGURIDAD Y PROPIEDAD
============================================================================
*/
#[derive(Debug, Clone)]
struct Vuelo {
    id: String,
    altitud: u32, // Clave principal del árbol AVL
}

/*
===============================================================================
¿POR QUÉ USAMOS Box<Nodo>?
===============================================================================

Rust necesita conocer el tamaño exacto de las estructuras en tiempo de compilación.
Un Nodo contiene otros Nodos recursivamente, por lo que usamos Box<Nodo>
para almacenar los hijos en el heap y mantener un tamaño fijo.

*/

struct Nodo {
    vuelo: Vuelo,
    izquierdo: Option<Box<Nodo>>,
    derecho: Option<Box<Nodo>>,
    altura: i32,
}

impl Nodo {
    fn nuevo(vuelo: Vuelo) -> Self {
        Nodo {
            vuelo,
            izquierdo: None,
            derecho: None,
            altura: 1,
        }
    }
}
/*
============================================================================
FUNCIONES AUXILIARES AVL
============================================================================
Obtiene la altura de un nodo.
`as_ref()`:
- Convierte `&Option<Box<Nodo>>` en `Option<&Box<Nodo>>`
- Permite leer el contenido SIN tomar ownership.
- Evita mover datos accidentalmente.

`map_or(0, |n| n.altura)`:
- Si existe nodo, retorna su altura.
- Si es None, retorna 0.
*/

fn obtener_altura(nodo: &Option<Box<Nodo>>) -> i32 {
    nodo.as_ref().map_or(0, |n| n.altura)
}

/*
Actualiza la altura del nodo usando:
1 + max(altura izquierda, altura derecha)
*/

fn actualizar_altura(nodo: &mut Nodo) {
    nodo.altura = 1 + std::cmp::max(
        obtener_altura(&nodo.izquierdo),
        obtener_altura(&nodo.derecho),
    );
}

/*
Factor de balance AVL:
balance = altura izquierda - altura derecha

Valores válidos:
-1, 0, 1

Si el valor sale de ese rango:
- Requiere rotación.

*/
fn obtener_balance(nodo: &Nodo) -> i32 {
    obtener_altura(&nodo.izquierdo) - obtener_altura(&nodo.derecho)
}
/*
============================================================================
ROTACIONES AVL Y SEGURIDAD DE MEMORIA
============================================================================

¿QUÉ OCURRE CON EL OWNERSHIP AL USAR `.take()`?

En Rust, Box<T> NO implementa Copy.

Esto significa que NO podemos copiar nodos automáticamente porque podrían
existir múltiples dueños del mismo dato en memoria, causando errores graves.

Por ejemplo, esto NO sería válido:

    let x = y.izquierdo;

porque intentaríamos mover el valor fuera de `y`.

`.take()` resuelve esto de forma segura:

    let x = y.izquierdo.take();

¿Qué hace `.take()`?

1. Extrae el valor del Option.
2. Transfiere ownership al nuevo dueño.
3. Deja `None` en el lugar original.

Resultado:
- Rust sabe exactamente quién es dueño del nodo.
- No existen referencias inválidas.
- No hay dobles liberaciones de memoria.
- El Borrow Checker permite reorganizar el árbol de forma segura.

Esto es fundamental durante las rotaciones AVL porque los nodos cambian
constantemente de posición.
*/

/*
Rotación derecha.

Se utiliza en desbalance:
IZQUIERDA - IZQUIERDA (LL)
*/

fn rotar_derecha(mut y: Box<Nodo>) -> Box<Nodo> {

    // Tomamos ownership del hijo izquierdo
    let mut x = y.izquierdo.take().expect("Error de radar");

    // El subárbol derecho de x pasa a ser hijo izquierdo de y
    y.izquierdo = x.derecho.take();

    actualizar_altura(&mut y);

    // y se convierte en hijo derecho de x
    x.derecho = Some(y);

    actualizar_altura(&mut x);

    // x es la nueva raíz del subárbol
    x
}

/*
Rotación izquierda.

Se utiliza en desbalance:
DERECHA - DERECHA (RR)
*/
fn rotar_izquierda(mut x: Box<Nodo>) -> Box<Nodo> {

    // Tomamos ownership del hijo derecho
    let mut y = x.derecho.take().expect("Error de radar");

    // El subárbol izquierdo de y pasa a ser hijo derecho de x
    x.derecho = y.izquierdo.take();

    actualizar_altura(&mut x);

    // x se convierte en hijo izquierdo de y
    y.izquierdo = Some(x);

    actualizar_altura(&mut y);

    // y es la nueva raíz del subárbol
    y
}

/*
============================================================================
FUNCIONES DE INSERCIÓN AVL
============================================================================
Inserta un nuevo vuelo en el árbol y lo balancea automáticamente.
*/

fn insertar(nodo_opt: Option<Box<Nodo>>, vuelo: Vuelo) -> Box<Nodo> {
    // 1. Guardamos la altitud en una variable local antes de mover 'vuelo'
    let altitud_nueva = vuelo.altitud;

    let mut nodo = match nodo_opt {
        None => return Box::new(Nodo::nuevo(vuelo)),
        Some(n) => n,
    };

    // 2. Aquí usamos altitud_nueva para comparar
    if altitud_nueva < nodo.vuelo.altitud {
        nodo.izquierdo = Some(insertar(nodo.izquierdo.take(), vuelo));
    } else if altitud_nueva > nodo.vuelo.altitud {
        nodo.derecho = Some(insertar(nodo.derecho.take(), vuelo));
    } else {
        return nodo; // Altitud duplicada
    }

    // Actualizar altura
    actualizar_altura(&mut nodo);

    // Obtener balance
    let balance = obtener_balance(&nodo);

    // 3. En las rotaciones, usamos altitud_nueva en lugar de vuelo.altitud
    
    // Caso Izquierda-Izquierda (LL)
    if balance > 1 && altitud_nueva < nodo.izquierdo.as_ref().unwrap().vuelo.altitud {
        return rotar_derecha(nodo);
    }

    // Caso Derecha-Derecha (RR)
    if balance < -1 && altitud_nueva > nodo.derecho.as_ref().unwrap().vuelo.altitud {
        return rotar_izquierda(nodo);
    }

    // Caso Izquierda-Derecha (LR)
    if balance > 1 && altitud_nueva > nodo.izquierdo.as_ref().unwrap().vuelo.altitud {
        let hijo_izq = nodo.izquierdo.take().unwrap();
        nodo.izquierdo = Some(rotar_izquierda(hijo_izq));
        return rotar_derecha(nodo);
    }

    // Caso Derecha-Izquierda (RL)
    if balance < -1 && altitud_nueva < nodo.derecho.as_ref().unwrap().vuelo.altitud {
        let hijo_der = nodo.derecho.take().unwrap();
        nodo.derecho = Some(rotar_derecha(hijo_der));
        return rotar_izquierda(nodo);
    }

    nodo
}



// ================================================
// PRUEBA DE ESCRITORIO AVL
// ================================================
/*
INSERCIONES:
[5000, 3000, 2000, 4000, 3500, 6000]
===================================================
PASO 1 — INSERTAR 5000
===================================================

        5000


Árbol balanceado.
===================================================
PASO 2 — INSERTAR 3000
===================================================

        5000
       /
    3000

Balance correcto.

===================================================
PASO 3 — INSERTAR 2000
===================================================

        5000
       /
    3000
    /
 2000

Desbalance en 5000:
Balance = +2

CASO:
IZQUIERDA - IZQUIERDA (LL)

Se aplica:
ROTACIÓN SIMPLE DERECHA

Resultado:

        3000
       /    \
    2000    5000


===================================================
PASO 4 — INSERTAR 4000
===================================================

        3000
       /    \
    2000    5000
            /
         4000

Árbol balanceado.

===================================================
PASO 5 — INSERTAR 3500
===================================================

        3000
       /    \
    2000    5000
            /
         4000
         /
      3500

Desbalance en 5000:
Balance = +2

CASO:
IZQUIERDA - IZQUIERDA (LL)

Se aplica:
ROTACIÓN SIMPLE DERECHA

Resultado:

         3000
        /    \
     2000    4000
             /   \
          3500   5000


===================================================
PASO 6 — INSERTAR 6000
===================================================

         3000
        /    \
     2000    4000
             /   \
          3500   5000
                     \
                     6000

Árbol balanceado.


ÁRBOL AVL FINAL
                 3000
               /      \
            2000      4000
                     /    \
                  3500    5000
                               \
                               6000


===================================================
ROTACIONES REALIZADAS
===================================================

1. Inserción de 2000
   - Rotación simple derecha
   - Caso Izquierda-Izquierda (LL)

2. Inserción de 3500
   - Rotación simple derecha
   - Caso Izquierda-Izquierda (LL)

*/

/*
============================================================================
    FASE 2 — LOCALIZACIÓN DE VUELOS
============================================================================
Busca un vuelo por su altitud dentro del árbol AVL.

Parámetros:
- nodo: referencia al nodo actual
- altitud: altitud del vuelo a buscar

Retorna:
- Some(&Vuelo): referencia al vuelo encontrado
- None: si el vuelo no existe

Características:
- Usa solo referencias (&), por lo que NO modifica el árbol.
- No realiza copias innecesarias.
- Complejidad O(log n) en un AVL balanceado.
*/

fn buscar_vuelo(
    nodo: &Option<Box<Nodo>>,
    altitud: u32
) -> Option<&Vuelo> {
    match nodo {
        None => None,
        Some(n) => {
            if altitud == n.vuelo.altitud {
                Some(&n.vuelo)
            } else if altitud < n.vuelo.altitud {
                buscar_vuelo(&n.izquierdo, altitud)
            } else {
                buscar_vuelo(&n.derecho, altitud)
            }
        }
    }
}

/*
============================================================================
    FASE 3 — DESCENSO Y ATERRIZAJE (ELIMINACIÓN AVL)
============================================================================

Encuentra el vuelo con mayor altitud dentro de un subárbol izquierdo.

Se utiliza como:
PREDECESOR IN-ORDER

El predecesor es:
- El valor más grande del subárbol izquierdo.
- Se usa para reemplazar un nodo con dos hijos.
*/

fn encontrar_maximo(nodo: &Box<Nodo>) -> &Vuelo {

    match &nodo.derecho {

        None => &nodo.vuelo,

        Some(der) => encontrar_maximo(der),
    }
}

/*
Elimina un vuelo del árbol AVL usando su altitud.

Casos manejados:

1. Nodo hoja
   → Se elimina directamente.

2. Nodo con un hijo
   → El nodo es reemplazado por su hijo.

3. Nodo con dos hijos
   → Se reemplaza usando el predecesor in-order
     (máximo del subárbol izquierdo).

Después de eliminar:
- Se recalculan alturas.
- Se aplican rotaciones AVL si es necesario.
*/

fn eliminar_vuelo(
    nodo_opt: Option<Box<Nodo>>,
    altitud: u32
) -> Option<Box<Nodo>> {

    let mut nodo = match nodo_opt {

        None => return None,

        Some(n) => n,
    };

    // ================================
    // BÚSQUEDA RECURSIVA
    // ================================

    if altitud < nodo.vuelo.altitud {

        nodo.izquierdo =
            eliminar_vuelo(nodo.izquierdo.take(), altitud);

    } else if altitud > nodo.vuelo.altitud {

        nodo.derecho =
            eliminar_vuelo(nodo.derecho.take(), altitud);

    } else {

        // ==============================
        // NODO ENCONTRADO
        // ==============================

        // Caso 1:
        // Nodo sin hijo izquierdo

        if nodo.izquierdo.is_none() {

            return nodo.derecho;
        }

        // Caso 2:
        // Nodo sin hijo derecho

        if nodo.derecho.is_none() {

            return nodo.izquierdo;
        }

        // Caso 3:
        // Nodo con dos hijos

        if let Some(ref izquierdo) = nodo.izquierdo {

            // Obtener predecesor in-order
            
            let predecesor = encontrar_maximo(izquierdo).clone();

            // Reemplazar datos del nodo

            nodo.vuelo = predecesor;

            // Eliminar el predecesor original

            nodo.izquierdo =
                eliminar_vuelo(
                    nodo.izquierdo.take(),
                    nodo.vuelo.altitud
                );
        }
    }

    // ========================================================================
    //   ACTUALIZAR ALTURA
    // ========================================================================

    actualizar_altura(&mut nodo);

    let balance = obtener_balance(&nodo);

    // ========================================================================
    //   RE-BALANCEO AVL
    // ========================================================================

    // Caso Izquierda-Izquierda (LL)

    if balance > 1 &&
        obtener_balance(nodo.izquierdo.as_ref().unwrap()) >= 0 {

        return Some(rotar_derecha(nodo));
    }

    // Caso Izquierda-Derecha (LR)

    if balance > 1 &&
        obtener_balance(nodo.izquierdo.as_ref().unwrap()) < 0 {

        let hijo_izq = nodo.izquierdo.take().unwrap();

        nodo.izquierdo =
            Some(rotar_izquierda(hijo_izq));

        return Some(rotar_derecha(nodo));
    }

    // Caso Derecha-Derecha (RR)

    if balance < -1 &&
        obtener_balance(nodo.derecho.as_ref().unwrap()) <= 0 {

        return Some(rotar_izquierda(nodo));
    }

    // Caso Derecha-Izquierda (RL)

    if balance < -1 &&
        obtener_balance(nodo.derecho.as_ref().unwrap()) > 0 {

        let hijo_der = nodo.derecho.take().unwrap();

        nodo.derecho =
            Some(rotar_derecha(hijo_der));

        return Some(rotar_izquierda(nodo));
    }

    Some(nodo)
}

/*

============================================================================
    FASE 4 — ALERTA DE PROXIMIDAD
============================================================================

Cuenta cuántos vuelos se encuentran dentro de un rango de altitud.

Parámetros:
- nodo: referencia al nodo actual
- min: altitud mínima
- max: altitud máxima

Retorna:
- Cantidad de vuelos dentro del rango indicado.

Optimización:
- Aprovecha la propiedad del árbol AVL.
- Evita recorrer ramas innecesarias.
- Complejidad aproximada O(log n) en casos balanceados.
*/

fn vuelos_en_rango(
    nodo: &Option<Box<Nodo>>,
    min: u32,
    max: u32
) -> usize {

    match nodo {

        None => 0,

        Some(n) => {

            // Si la altitud actual es menor al mínimo,
            // solo buscamos en el subárbol derecho

            if n.vuelo.altitud < min {

                vuelos_en_rango(&n.derecho, min, max)

            // Si la altitud actual es mayor al máximo,
            // solo buscamos en el subárbol izquierdo

            } else if n.vuelo.altitud > max {

                vuelos_en_rango(&n.izquierdo, min, max)

            } else {

                // El vuelo actual está dentro del rango

                1
                + vuelos_en_rango(&n.izquierdo, min, max)
                + vuelos_en_rango(&n.derecho, min, max)
            }
        }
    }
}


fn main() {

    let mut radar: Option<Box<Nodo>> = None;
/*
    ========================================================================
    INSERCIÓN DE VUELOS
    ========================================================================
*/
    let datos = vec![
        ("AV123", 5000),
        ("UA456", 3000),
        ("IB101", 2000),
        ("AF999", 4000),
        ("TA222", 3500),
        ("AM777", 6000),
    ];

    println!("=== MOTOR DE TRÁFICO AÉREO (AVL) ===\n");

    println!("Insertando vuelos...\n");

    for (id, alt) in datos {

        let vuelo = Vuelo {
            id: id.to_string(),
            altitud: alt,
        };

        radar = Some(insertar(radar.take(), vuelo));
    }
/* 
    ========================================================================
    FASE 2 — BÚSQUEDA
    ========================================================================
*/
    println!("=== FASE 2: BÚSQUEDA DE VUELOS ===\n");

    // Vuelo existente
    match buscar_vuelo(&radar, 4000) {

        Some(vuelo) => {
            println!(
                "Vuelo encontrado -> ID: {}, Altitud: {}",
                vuelo.id,
                vuelo.altitud
            );
        }

        None => {
            println!("No se encontró el vuelo.");
        }
    }

    // Vuelo inexistente
    match buscar_vuelo(&radar, 9000) {

        Some(vuelo) => {
            println!(
                "Vuelo encontrado -> ID: {}, Altitud: {}",
                vuelo.id,
                vuelo.altitud
            );
        }

        None => {
            println!("No existe un vuelo con altitud 9000.");
        }
    }

/*
    ========================================================================
    FASE 3 — ELIMINACIÓN
    ========================================================================
*/
    println!("\n=== FASE 3: ELIMINACIÓN DE VUELOS ===\n");

    // Eliminar nodo hoja
    println!("Eliminando vuelo con altitud 2000...");
    radar = eliminar_vuelo(radar.take(), 2000);

    // Verificación
    match buscar_vuelo(&radar, 2000) {

        Some(_) => println!("Error: el vuelo aún existe."),

        None => println!("Vuelo eliminado correctamente."),
    }

    // Eliminar nodo con dos hijos
    println!("\nEliminando vuelo con altitud 4000...");
    radar = eliminar_vuelo(radar.take(), 4000);

    // Verificación
    match buscar_vuelo(&radar, 4000) {

        Some(_) => println!("Error: el vuelo aún existe."),

        None => println!("Vuelo eliminado correctamente."),
    }
/* 
    ========================================================================
    FASE 4 — ALERTA DE PROXIMIDAD
    ========================================================================
*/
    println!("\n=== FASE 4: ALERTA DE PROXIMIDAD ===\n");

    let cantidad = vuelos_en_rango(&radar, 3000, 6000);

    println!(
        "Cantidad de vuelos entre 3000 y 6000 pies: {}",
        cantidad
    );

    println!("\n=== SIMULACIÓN FINALIZADA ===");
}
