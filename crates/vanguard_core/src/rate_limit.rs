use dashmap::DashMap;
use std::net::IpAddr;
use std::time::Instant;

#[derive(Clone)]
pub struct RateLimiter {
    // Map IP address to (tokens, last_refill)
    buckets: DashMap<IpAddr, (u32, Instant)>,
    capacity: u32,
    refill_rate: f32, // Tokens per second
}

impl RateLimiter {
    pub fn new(capacity: u32, refill_rate: f32) -> Self {
        Self {
            buckets: DashMap::new(),
            capacity,
            refill_rate,
        }
    }

    pub fn check_limit(&self, ip: IpAddr) -> bool {
        let mut entry = self.buckets.entry(ip).or_insert((self.capacity, Instant::now()));
        let now = Instant::now();
        
        let (mut tokens, last_refill) = *entry.value();
        
        let elapsed = now.duration_since(last_refill).as_secs_f32();
        let add_tokens = (elapsed * self.refill_rate) as u32;
        
        if add_tokens > 0 {
            tokens = std::cmp::min(self.capacity, tokens + add_tokens);
            entry.value_mut().1 = now;
        }

        if tokens > 0 {
            entry.value_mut().0 = tokens - 1;
            true // Allowed
        } else {
            false // Rate limited
        }
    }
}
