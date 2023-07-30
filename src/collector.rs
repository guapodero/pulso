use std::collections::HashMap;

use anyhow::Result;

use crate::capture::{ExtractedHeaders, PacketOwned};
use crate::sensitive::IpAddress;

#[derive(Default)]
pub struct Collector {
    connection_count: u64,
    connections: HashMap<(IpAddress, u16), u64>,
}

impl Collector {
    pub fn process(&mut self, packet: PacketOwned) -> Result<u64> {
        let ExtractedHeaders {
            source_ip,
            dest_port,
            ..
        } = packet.headers()?;

        self.connection_count += 1;

        self.connections
            .entry((source_ip, dest_port))
            .and_modify(|e| *e += 1)
            .or_insert(1);

        Ok(self.connection_count)
    }

    pub fn summary(&self) -> Vec<String> {
        let mut grouped: HashMap<IpAddress, Vec<(u16, i64)>> = HashMap::new();
        for ((source_ip, dest_port), count) in &self.connections {
            let port_count = (*dest_port, *count as i64); // i64 for descending sort
            grouped
                .entry(*source_ip)
                .and_modify(|counts| counts.push(port_count))
                .or_insert(vec![port_count]);
        }

        let mut sorted: Vec<(IpAddress, i64, Vec<(u16, i64)>)> = grouped
            .into_iter()
            .map(|(source_ip, mut counts)| {
                counts.sort_by_key(|t| t.0); // 2. ports ascending
                counts.sort_by_key(|t| -t.1); // 1. count descending
                let sum = counts.iter().fold(0, |acc, (_, c)| acc + c);
                (source_ip, sum, counts)
            })
            .collect();
        sorted.sort_by_key(|t| -t.1); // sum descending

        sorted
            .into_iter()
            .map(|(source_ip, sum, counts)| {
                let counts: Vec<String> = counts
                    .into_iter()
                    .map(|(k, v)| format!("{k}:{v}"))
                    .collect();
                format!("{}:{} {}", source_ip.hmac_hex(), sum, counts.join(" "))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::collector::Collector;
    use crate::sensitive::IpAddress;

    #[test]
    fn test_summary_grouped_sorted() {
        let ip1 = IpAddress::V6(1u128.swap_bytes().to_ne_bytes());
        let ip2 = IpAddress::V6(2u128.swap_bytes().to_ne_bytes());

        let collector = Collector {
            connection_count: 5,
            connections: HashMap::from([
                ((ip1, 123), 1),
                ((ip1, 234), 1),
                ((ip2, 345), 1),
                ((ip2, 456), 2),
            ]),
        };

        assert_eq!(
            collector.summary(),
            vec![
                format!("{}:3 456:2 345:1", ip2.hmac_hex()),
                format!("{}:2 123:1 234:1", ip1.hmac_hex()),
            ]
        );
    }
}
