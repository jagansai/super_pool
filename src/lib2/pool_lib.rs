use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

// ObjectPool struct that take any generic type of Arc of Vec![T]
pub struct ObjectPool<T>
where
    T: Default,
{
    // default_value: Arc<T>,
    pool: Arc<Mutex<Vec<Arc<T>>>>,
}

impl<T> ObjectPool<T>
where
    T: Default,
{
    pub fn new(data: Vec<Arc<T>>, default_value: T) -> Self {
        ObjectPool {
            pool: Arc::new(Mutex::new(data)),
            // default_value: Arc::new(default_value),
        }
    }

    fn get(&self) -> PooledObject<T>
    where
        T: Default,
    {
        let data = self.pool.lock().unwrap().pop();
        PooledObject {
            pool: Arc::clone(&self.pool),
            value: data.unwrap_or_else(|| {
                let tmp = Arc::new(T::default());
                self.pool.lock().unwrap().push(tmp.clone());
                Arc::clone(&tmp)
            }),
        }
    }
}

struct PooledObject<T>
where
    T: Default,
{
    value: Arc<T>,
    pool: Arc<Mutex<Vec<Arc<T>>>>,
}

impl<T> Drop for PooledObject<T>
where
    T: Default,
{
    fn drop(&mut self) {
        let mut pool = self.pool.lock().unwrap();
        pool.push(Arc::clone(&self.value));
    }
}

struct SuperPool<T>
where
    T: Default,
{
    pool_map: HashMap<String, ObjectPool<T>>,
}

impl<T> SuperPool<T>
where
    T: Default,
{
    pub fn new(pool_id: String, object_pool: ObjectPool<T>) -> Self {
        let mut pool = SuperPool {
            pool_map: HashMap::new(),
        };

        pool.pool_map.insert(pool_id, object_pool);
        pool
    }

    pub fn add(&mut self, pool_id: &str, object_pool: ObjectPool<T>) {
        self.pool_map.insert(pool_id.to_string(), object_pool);
    }

    pub fn get_data(&mut self, pool_id: &str) -> Option<PooledObject<T>> {
        self.pool_map
            .get(pool_id)
            .map(|object_pool| object_pool.get())
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

    println!("Demo .. pool_lib.rs");
    
    let mut pool = SuperPool::new(
        String::from("String"),
        ObjectPool::new(get_string_data(), String::from("default")),
    );

    {
        let v1 = pool.get_data("String");
        print_me(&v1);
        let v2 = pool.get_data("String");
        print_me(&v2);
    }
    let v3 = pool.get_data("String");
    print_me(&v3); // prints "pool" as `v1` went out of scope and the object got added back to the pool.

    // pool.add("Int", ObjectPool::new(get_nums(), 0));
}

fn print_me(v: &Option<PooledObject<String>>) {
    match v {
        Some(p) => println!("Got {:?}", p.value),
        None => println!("Got nothing..."),
    }
}
