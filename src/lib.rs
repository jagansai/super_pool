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
