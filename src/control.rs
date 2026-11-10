//! Hardware-independent transactions, exercised with an explicitly simulated test transport.
use crate::protocol::{CHANNEL, Channel, POWER, PowerCommand, Status};
use anyhow::Result;
use std::time::Duration;
use uuid::Uuid;
pub trait Transport {
    async fn write(&self, characteristic: Uuid, byte: u8, acknowledged: bool) -> Result<()>;
    async fn read(&self, characteristic: Uuid) -> Result<Vec<u8>>;
}
pub struct PowerOutcome {
    pub observed: Status,
    pub confirmed: bool,
}
pub async fn power(
    io: &impl Transport,
    command: PowerCommand,
    settle: Duration,
) -> Result<PowerOutcome> {
    io.write(POWER, command as u8, false).await?;
    let mut last_error = None;
    let mut observed = Status::Unavailable;
    for _ in 0..3 {
        tokio::time::sleep(settle).await;
        match io.read(POWER).await.and_then(|bytes| Status::parse(&bytes)) {
            Ok(status) => {
                last_error = None;
                observed = status;
                if observed.matches(command) {
                    return Ok(PowerOutcome {
                        observed,
                        confirmed: true,
                    });
                }
            }
            Err(error) => last_error = Some(error),
        }
    }
    if let Some(error) = last_error {
        return Err(error);
    }
    Ok(PowerOutcome {
        observed,
        confirmed: false,
    })
}
pub async fn channel(io: &impl Transport, value: Channel, settle: Duration) -> Result<Channel> {
    io.write(CHANNEL, value.byte(), true).await?;
    tokio::time::sleep(settle).await;
    Channel::parse(&io.read(CHANNEL).await?)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    struct SimulatedTransport {
        writes: RefCell<Vec<(Uuid, u8, bool)>>,
        reads: RefCell<std::collections::VecDeque<Vec<u8>>>,
        fail: bool,
    }
    impl SimulatedTransport {
        fn new(reads: Vec<Vec<u8>>) -> Self {
            Self {
                writes: RefCell::new(vec![]),
                reads: RefCell::new(reads.into()),
                fail: false,
            }
        }
    }
    impl Transport for SimulatedTransport {
        async fn write(&self, id: Uuid, byte: u8, ack: bool) -> Result<()> {
            if self.fail {
                anyhow::bail!("SIMULATED write failure");
            }
            self.writes.borrow_mut().push((id, byte, ack));
            Ok(())
        }
        async fn read(&self, _: Uuid) -> Result<Vec<u8>> {
            self.reads
                .borrow_mut()
                .pop_front()
                .ok_or_else(|| anyhow::anyhow!("SIMULATED missing response"))
        }
    }
    #[tokio::test]
    async fn power_write_is_not_confirmation() {
        let io = SimulatedTransport::new(vec![vec![0], vec![0], vec![0]]);
        let result = power(&io, PowerCommand::On, Duration::ZERO).await.unwrap();
        assert!(!result.confirmed);
        assert_eq!(result.observed, Status::Sleep);
        assert_eq!(*io.writes.borrow(), vec![(POWER, 1, false)]);
    }
    #[tokio::test]
    async fn delayed_confirmation_and_unknown() {
        let io = SimulatedTransport::new(vec![vec![255], vec![0], vec![11]]);
        assert!(
            power(&io, PowerCommand::On, Duration::ZERO)
                .await
                .unwrap()
                .confirmed
        );
        assert_eq!(io.writes.borrow().len(), 1);
    }
    #[tokio::test]
    async fn wake_command_is_not_the_readback_byte() {
        for byte in [1, 9, 11] {
            let io = SimulatedTransport::new(vec![vec![byte]]);
            let result = power(&io, PowerCommand::On, Duration::ZERO).await.unwrap();
            assert!(result.confirmed);
            assert_eq!(result.observed, Status::On(byte));
            assert_eq!(*io.writes.borrow(), vec![(POWER, 1, false)]);
        }
        let io = SimulatedTransport::new(vec![vec![12]; 3]);
        let result = power(&io, PowerCommand::On, Duration::ZERO).await.unwrap();
        assert!(!result.confirmed);
        assert_eq!(result.observed, Status::Unknown(12));
    }
    #[tokio::test]
    async fn empty_read_never_confirms() {
        let io = SimulatedTransport::new(vec![vec![], vec![], vec![]]);
        assert!(
            power(&io, PowerCommand::Sleep, Duration::ZERO)
                .await
                .is_err()
        );
    }
    #[tokio::test]
    async fn channel_ack_and_actual_readback() {
        for n in [1, 5, 16] {
            let io = SimulatedTransport::new(vec![vec![n]]);
            assert_eq!(
                channel(&io, Channel::new(n).unwrap(), Duration::ZERO)
                    .await
                    .unwrap()
                    .byte(),
                n
            );
            assert_eq!(*io.writes.borrow(), vec![(CHANNEL, n, true)]);
        }
        let io = SimulatedTransport::new(vec![vec![9]]);
        assert_eq!(
            channel(&io, Channel::new(5).unwrap(), Duration::ZERO)
                .await
                .unwrap()
                .byte(),
            9
        );
    }
    #[tokio::test]
    async fn one_failed_device_does_not_stop_group() {
        let mut failed = SimulatedTransport::new(vec![]);
        failed.fail = true;
        let healthy = SimulatedTransport::new(vec![vec![11]]);
        let mut results = vec![];
        for device in [&failed, &healthy] {
            results.push(power(device, PowerCommand::On, Duration::ZERO).await);
        }
        assert!(results[0].is_err());
        assert!(results[1].as_ref().unwrap().confirmed);
        assert!(failed.writes.borrow().is_empty());
        assert_eq!(healthy.writes.borrow().len(), 1);
    }
}
