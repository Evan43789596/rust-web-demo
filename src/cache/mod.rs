use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};
use redis::Commands;
use redis::RedisResult;
use redis::ToRedisArgs;
use redis::FromRedisValue;
use redis::RedisError;
use redis::Value;
use std::io::Error;
use std::io::ErrorKind;
use rustc_serialize::{Encodable, Decodable};
use r2d2_redis::RedisConnectionManager;
use std::default::Default;
use r2d2::{Pool, PooledConnection};

pub fn set<T: Encodable>(key: &str, value: &T) -> RedisResult<()> {
    let _: () = try!(get_conn().set(key, EncodeWrapper(value)));
    Ok(())
}

pub fn get<T: Decodable>(key: &str) -> RedisResult<T> {
    let t: DecodeWrapper<T> = try!(get_conn().get(key));
    Ok(t.0)
}


pub fn del(key: &str) -> RedisResult<()> {
    let _: () = try!(get_conn().del(key));
    Ok(())
}

struct EncodeWrapper<'a, T: 'a>(&'a T);
impl<'a, T: Encodable + 'a> ToRedisArgs for EncodeWrapper<'a, T> {
    fn to_redis_args(&self) -> Vec<Vec<u8>> {
        vec![encode(self.0, SizeLimit::Infinite).unwrap()]
    }
}
struct DecodeWrapper<T>(T);
impl<T: Decodable> FromRedisValue for DecodeWrapper<T> {
    fn from_redis_value(v: &Value) -> RedisResult<DecodeWrapper<T>> {
        if let Value::Data(ref items) = *v {
            match decode(&items[..]) {
                Ok(decoded) => {
                    return Ok(DecodeWrapper(decoded));
                }
                Err(err) => {
                    panic!("erro read redis cache:{}", err);
                }
            }
        }
        Err(RedisError::from(Error::new(ErrorKind::Other, "oh no!")))
    }
}


lazy_static! {
    static ref POOL:Pool<RedisConnectionManager>  = connect_pool();
}
fn connect_pool() -> Pool<RedisConnectionManager> {
    let config = Default::default();
    let connect_str = "redis://localhost";
    info!("Connecting to {}", connect_str);
    let manager = RedisConnectionManager::new(connect_str).unwrap();
    // let pool = Pool::new(config, manager).unwrap();
    match Pool::new(config, manager) {
        Ok(pool) => {
            info!("Connected to redis with pool: {:?}", pool);
            return pool;
        }
        Err(err) => {
            panic!("Error occured when connectted to redis:{}. Error info:{}",
                   connect_str,
                   err);
        }
    };
}

fn get_conn() -> PooledConnection<RedisConnectionManager> {
    let conn = POOL.get().unwrap();
    conn
}
