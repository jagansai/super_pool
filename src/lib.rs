use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum ReusableEnum {
    Int(i32),
    Text(String),
}

#[derive(Debug, Clone)]
pub struct ObjectPool {
    values: Arc<Mutex<Vec<ReusableEnum>>>,
    default_value: ReusableEnum,
}

impl ObjectPool {
    pub fn new(values: Vec<ReusableEnum>, default_value: ReusableEnum) -> Self {
        ObjectPool {
            values: Arc::new(Mutex::new(values)),
            default_value: default_value,
        }
    }

    pub fn get(&self) -> Option<PooledObject> {
        let mut values = self.values.lock().unwrap();
        if let Some(value) = values.pop() {
            Some(PooledObject {
                value: Some(value),
                pool: Arc::clone(&self.values),
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct PooledObject {
    value: Option<ReusableEnum>,
    pool: Arc<Mutex<Vec<ReusableEnum>>>,
}

impl Drop for PooledObject {
    fn drop(&mut self) {
        if let Some(value) = self.value.take() {
            self.pool.lock().unwrap().push(value);
        }
    }
}

impl std::ops::Deref for PooledObject {
    type Target = ReusableEnum;

    fn deref(&self) -> &Self::Target {
        self.value.as_ref().unwrap()
    }
}

pub struct SuperPool {
    pool: HashMap<String, ObjectPool>,
}

impl SuperPool {
    pub fn new_with_pool(pool_type: String, object_pool: ObjectPool) -> Self {
        let mut tmp = SuperPool {
            pool: HashMap::new(),
        };

        tmp.pool.insert(pool_type, object_pool);
        tmp
    }

    pub fn add_to_pool(&mut self, pool_type: String, object_pool: ObjectPool) {
        self.pool.insert(pool_type, object_pool);
    }

    pub fn get(&mut self, pool_type: &str) -> Option<PooledObject> {
        if let Some(object_pool) = self.pool.get(pool_type) {
            if let Some(pooled_object) = object_pool.get() {
                Some(pooled_object)
            } else {
                // If the pool is empty, add the default value and return it
                object_pool
                    .values
                    .lock()
                    .unwrap()
                    .push(object_pool.default_value.clone());
                object_pool.get()
            }
        } else {
            None
        }
    }

    pub fn get_len(&self, pool_type: &str) -> usize {
        self.pool
            .get(pool_type)
            .map(|pool| pool.values.lock().unwrap().len())
            .unwrap_or(0)
    }
}

pub fn demo() {
    let obj_pool1 = ObjectPool::new(
        vec![
            ReusableEnum::Text(String::from("this")),
            ReusableEnum::Text(String::from("is")),
            ReusableEnum::Text(String::from("a")),
            ReusableEnum::Text(String::from("pool")),
        ],
        ReusableEnum::Text(String::from("Default")),
    );

    let obj_pool2 = ObjectPool::new(
        vec![
            ReusableEnum::Int(1),
            ReusableEnum::Int(2),
            ReusableEnum::Int(3),
        ],
        ReusableEnum::Int(0),
    );

    let mut super_pool: SuperPool = SuperPool::new_with_pool(String::from("String"), obj_pool1);
    super_pool.add_to_pool(String::from("int"), obj_pool2);

    string_demo(&mut super_pool);

    int_demo(&mut super_pool);
}

fn int_demo(super_pool: &mut SuperPool) {
    let iv1 = super_pool.get("int");
    let iv2 = super_pool.get("int");
    let iv3 = super_pool.get("int");
    print_me("int", &iv1);
    print_me("int", &iv2);
    print_me("int", &iv3);
    // now we exchausted the items in the pool.
    print_me("int", &super_pool.get("int")); // returns 0.
}

fn string_demo(super_pool: &mut SuperPool) {
    {
        let v1 = super_pool.get("String");
        let v2 = super_pool.get("String");
        print_me("String", &v1);
        print_me("String", &v2);
    }
    // now if we get a value from the pool, we should be getting one of "a"/"pool".
    // This is because, at this point v1 and v2 are out of scope and they should be added back to the pool.
    let v3 = super_pool.get("String");
    print_me("String", &v3);
    let v4 = super_pool.get("String");
    print_me("String", &v4);
    let v5 = super_pool.get("String");
    print_me("String", &v5);
    let v6 = super_pool.get("String");
    print_me("String", &v6);
}

fn print_me(pool_type: &str, v2: &Option<PooledObject>) {
    match v2 {
        Some(x) => println!("For pool_type({}), got the value: {:?}", pool_type, *x),
        None => println!("Got None"),
    }
}
