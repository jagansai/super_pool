use std::{
    any::Any,
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, Mutex},
};

// Introduce a new trait that is called AnyPool.
// Trait for dynamic downcasting
trait AnyPool {
    fn as_any(&self) -> &dyn std::any::Any;
}

// ObjectPool struct that take any generic type of Arc of Vec![T]
pub struct ObjectPool<T>
where
    T: Default,
{
    pool: Arc<Mutex<Vec<Arc<T>>>>,
}


impl<T: 'static> AnyPool for ObjectPool<T>
where
    T: Default,
{
    fn as_any(&self) -> &dyn std::any::Any {
        self // this has temporary lifetime. So the type needs to be qualified with 'static.
    }
}

impl<T> ObjectPool<T>
where
    T: Default,
{
    pub fn new(data: Vec<Arc<T>>) -> Self {
        ObjectPool {
            pool: Arc::new(Mutex::new(data)),
        }
    }

    fn get(&self) -> PooledObject<T> { // PooledObject is declared below. This holds the object that just got returned from the pool.
        let data = self.pool.lock().unwrap().pop();
        PooledObject {
            pool: Arc::clone(&self.pool),
            value: data.unwrap_or_else(|| {
                let tmp = Arc::new(T::default());
                self.pool.lock().unwrap().push(tmp.clone());
                tmp
            }),
        }
    }
}

struct PooledObject<T> { // holds the reference count of both the value that is taken from the pool and the pool as well.
    value: Arc<T>,
    pool: Arc<Mutex<Vec<Arc<T>>>>,
}

/*
Drop trait is implemented for PooledObject so that when the PooledObject goes out of scope
we push the object back to the pool. No more manual pushing back to the pool.
*/ 
impl<T> Drop for PooledObject<T> {
    fn drop(&mut self) {
        let mut pool = self.pool.lock().unwrap();
        pool.push(Arc::clone(&self.value));
    }
}

/*
    This class holds the map for the pool id and the ObjectPool.
    We can't have a generic type here. So take the value as Box<dyn AnyPool>
*/
struct SuperPool {
    pool_map: HashMap<String, Arc<Box<dyn AnyPool>>>,
}

impl SuperPool {
    pub fn new() -> Self {
        SuperPool {
            pool_map: HashMap::new(),
        }
    }

    pub fn add_pool<T: 'static>(&mut self, pool_id: &str, object_pool: ObjectPool<T>)
    where
        T: Default,
    {
        self.pool_map
            .insert(pool_id.to_string(), Arc::new(Box::new(object_pool)));
    }

    pub fn get_data<T: 'static>(&mut self, pool_id: &str) -> Option<PooledObject<T>>
    where
        T: Default,
    {
        if let Some(pool) = self.pool_map.get(pool_id) {
            let tmp = pool.as_any().downcast_ref::<ObjectPool<T>>();
            if let Some(pool) = tmp {
                return Some(pool.get());
            }
        }
        None
    }
}

fn get_string_data() -> Vec<Arc<String>> {
    vec![
        Arc::new(String::from("This")),
        Arc::new(String::from("is")),
        Arc::new(String::from("a")),
        Arc::new(String::from("pool")),
    ]
}

fn get_nums() -> Vec<Arc<i32>> {
    vec![Arc::new(1), Arc::new(2), Arc::new(3)]
}

pub fn demo() {
    println!("Demo .. pool_lib2.rs");
    let mut pool = SuperPool::new();
    pool.add_pool(
        "String",
        ObjectPool::new(get_string_data()),
    );
    pool.add_pool("Int", ObjectPool::new(get_nums()));

    {
        let v1 = pool.get_data::<String>("String");
        print_me(&v1);
        let v2 = pool.get_data::<String>("String");
        print_me(&v2);
    }
    let v3 = pool.get_data::<String>("String");
    print_me(&v3); // v1 and v2 out of scope. So they will get added back to the ObjectPool. This prints "pool" again.


    let iv1 = pool.get_data::<i32>("Int");
    print_me(&iv1);
}

fn print_me<T: Debug + Any>(v: &Option<PooledObject<T>>) {
    match v {
        Some(p) => println!("Got {:?}", p.value),
        None => println!("Got nothing..."),
    }
}
