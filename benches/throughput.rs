#![feature(test)]

extern crate mpsc;
extern crate rand;
extern crate test;

use std::sync::Arc;
use std::sync::mpsc as std_mpsc;
use std::thread;

use rand::{thread_rng, Rng};
use test::Bencher;

fn get_data() -> Vec<u8> {
    thread_rng().gen_iter().take(1000).collect()
}

fn get_data_sum<I: IntoIterator<Item=u8>>(xs: I) -> u64 {
    xs.into_iter().fold(0, |sum, x| sum + (x as u64))
}

macro_rules! bench_chan_spsc {
    ($name:ident, $cons:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            let data = Arc::new(get_data());
            let sum = get_data_sum(data.iter().cloned());
            b.bytes = data.len() as u64;
            b.iter(|| {
                let data = data.clone();
                let (s, r) = $cons;
                thread::spawn(move || {
                    for &datum in &*data {
                        let _ = s.send(datum);
                    }
                });
                assert_eq!(sum, get_data_sum(r.iter()));
            });
        }
    }
}

bench_chan_spsc!(spsc_chan_sync_unbuffered, mpsc::sync_channel(0));
bench_chan_spsc!(spsc_chan_sync_buffered, mpsc::sync_channel(1));
bench_chan_spsc!(spsc_chan_sync_buffered_all, mpsc::sync_channel(1000));
bench_chan_spsc!(spsc_chan_async, mpsc::channel());
bench_chan_spsc!(std_spsc_chan_sync_unbuffered, std_mpsc::sync_channel(0));
bench_chan_spsc!(std_spsc_chan_sync_buffered, std_mpsc::sync_channel(1));
bench_chan_spsc!(std_spsc_chan_sync_buffered_all, std_mpsc::sync_channel(1000));
bench_chan_spsc!(std_spsc_chan_async, std_mpsc::channel());

macro_rules! bench_chan_mpsc {
    ($name:ident, $cons:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            let data = Arc::new(get_data());
            let sum = get_data_sum(data.iter().cloned());
            b.bytes = data.len() as u64;
            b.iter(|| {
                let (s, r) = $cons;
                for i in 0..4 {
                    let data = data.clone();
                    let s = s.clone();
                    let start = i * (data.len() / 4);
                    let end = start + (data.len() / 4);
                    thread::spawn(move || {
                        for &datum in &(&*data)[start..end] {
                            let _ = s.send(datum);
                        }
                    });
                }
                drop(s);
                assert_eq!(sum, get_data_sum(r.iter()));
            });
        }
    }
}

bench_chan_mpsc!(mpsc_chan_sync_unbuffered, mpsc::sync_channel(0));
bench_chan_mpsc!(mpsc_chan_sync_buffered, mpsc::sync_channel(1));
bench_chan_mpsc!(mpsc_chan_sync_buffered_all, mpsc::sync_channel(1000));
bench_chan_mpsc!(mpsc_chan_async, mpsc::channel());
bench_chan_mpsc!(std_mpsc_chan_sync_unbuffered, std_mpsc::sync_channel(0));
bench_chan_mpsc!(std_mpsc_chan_sync_buffered, std_mpsc::sync_channel(1));
bench_chan_mpsc!(std_mpsc_chan_sync_buffered_all, std_mpsc::sync_channel(1000));
bench_chan_mpsc!(std_mpsc_chan_async, std_mpsc::channel());
