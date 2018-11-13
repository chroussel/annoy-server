#![feature(duration_as_u128)]
extern crate annoy_rs;
extern crate rand;

use annoy_rs::annoy::*;
use rand::distributions::Standard;
use rand::prelude::*;
use rand::Rng;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::SystemTime;

fn main() {
    let n: i32 = 100000;
    const f: usize = 40;

    let mut rng = thread_rng();
    let mut indexBuilder = AnnoyIndexBuilder::new(f as i32);

    for i in 0..n {
        let mut arr: Vec<f32> = rng.sample_iter(&Standard).take(f).collect();
        indexBuilder.add_item(i, arr.as_slice())
    }

    let index = indexBuilder.build(Some(2 * f as i32));

    let limits = &[10, 100, 1000, 10000];
    let k = 10;
    let mut prec_sum = HashMap::new();
    let prec_n = 1000;
    let mut time_sum = HashMap::new();

    for i in 0..prec_n {
        let j = rng.gen_range(0, n);
        let (closest, _) = index.get_nns_by_item(j, k, Some(n));
        let closest_set: HashSet<_> = closest.iter().collect();
        for limit in limits.iter() {
            let t0 = SystemTime::now();
            let (toplist, _) = index.get_nns_by_item(j, k, Some(*limit));
            let T = t0.elapsed().unwrap();
            let toplistSet: HashSet<_> = toplist.iter().collect();

            let found = closest_set.intersection(&toplistSet).count();
            let hitrate = 1.0 * (found as f32) / (k as f32);
            prec_sum.insert(limit, prec_sum.get(limit).unwrap_or(&0.0) + hitrate);
            time_sum.insert(limit, time_sum.get(limit).unwrap_or(&0) + T.as_nanos());
        }
    }

    for limit in limits.iter() {
        let prec = 100.0 * prec_sum.get(limit).unwrap() / (prec_n + 1) as f32;
        let avg_time = time_sum.get(limit).unwrap() / (prec_n + 1);
        println!(
            "limit: {:>6} - precision: {:.6}% - avg time: {} ns",
            limit, prec, avg_time
        )
    }
}

/*
for i in xrange(prec_n):
    j = random.randrange(0, n)

    closest = set(t.get_nns_by_item(j, k, n))
    for limit in limits:
        t0 = time.time()
        toplist = t.get_nns_by_item(j, k, limit)
        T = time.time() - t0

        found = len(closest.intersection(toplist))
        hitrate = 1.0 * found / k
        prec_sum[limit] = prec_sum.get(limit, 0.0) + hitrate
        time_sum[limit] = time_sum.get(limit, 0.0) + T

for limit in limits:
    print('limit: %-9d precision: %6.2f%% avg time: %.6fs'
          % (limit, 100.0 * prec_sum[limit] / (i + 1),
             time_sum[limit] / (i + 1)))
*/
