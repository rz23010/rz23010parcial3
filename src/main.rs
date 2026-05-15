// ============================================================================
// FASE 1 — ANÁLISIS DE SEGURIDAD Y PROPIEDAD
// ============================================================================

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

