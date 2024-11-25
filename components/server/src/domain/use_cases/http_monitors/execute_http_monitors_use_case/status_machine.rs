use crate::domain::entities::http_monitor::HttpMonitorStatus;

// Determines the next status of an HTTP monitor based on its current state and the latest ping result.
/// Returns a tuple of the new status counter and the new status.
pub fn next_status(
    downtime_confirmation_threshold: i16,
    recovery_confirmation_threshold: i16,
    current_status: HttpMonitorStatus,
    current_status_counter: i16,
    last_ping_ok: bool,
) -> (i16, HttpMonitorStatus) {
    match (current_status, last_ping_ok) {
        // Transition from archived should not change the status
        // An archived monitor is not monitored anymore and should never go through the status machine
        (HttpMonitorStatus::Archived, _) => unreachable!("try to compute the next status of an archived monitor"),
        // Down monitor staying down
        (HttpMonitorStatus::Down, false) => (
            current_status_counter.saturating_add(1),
            HttpMonitorStatus::Down,
        ),
        // Transition from down to recovering
        (HttpMonitorStatus::Down, true) if recovery_confirmation_threshold > 1 => {
            (1, HttpMonitorStatus::Recovering)
        }
        // Transition from down/unknown/inactive to up (no confirmation)
        (HttpMonitorStatus::Down | HttpMonitorStatus::Unknown | HttpMonitorStatus::Inactive, true) => {
            (1, HttpMonitorStatus::Up)
        }
        // Transition from suspicious to down
        (HttpMonitorStatus::Suspicious, false) => {
            let next_status_counter = current_status_counter.saturating_add(1);
            if next_status_counter >= downtime_confirmation_threshold {
                (1, HttpMonitorStatus::Down)
            } else {
                (next_status_counter, HttpMonitorStatus::Suspicious)
            }
        }
        // Transition from suspicious to recovering
        (HttpMonitorStatus::Suspicious, true) => (1, HttpMonitorStatus::Recovering),
        // Transition from recovering back to suspicious
        (HttpMonitorStatus::Recovering, false) if downtime_confirmation_threshold > 1 => {
            (1, HttpMonitorStatus::Suspicious)
        }
        // Transition from recovering to down (no confirmation)
        (HttpMonitorStatus::Recovering, false) => (1, HttpMonitorStatus::Down),
        // Transition from recovering to up
        (HttpMonitorStatus::Recovering, true) => {
            let next_status_counter = current_status_counter.saturating_add(1);
            if next_status_counter >= recovery_confirmation_threshold {
                (1, HttpMonitorStatus::Up)
            } else {
                (next_status_counter, HttpMonitorStatus::Recovering)
            }
        }
        // Transition from up/unknown/inactive to suspicious
        (HttpMonitorStatus::Up | HttpMonitorStatus::Unknown | HttpMonitorStatus::Inactive, false)
            if downtime_confirmation_threshold > 1 =>
        {
            (1, HttpMonitorStatus::Suspicious)
        }
        // Transition from up to down (no confirmation)
        (HttpMonitorStatus::Up | HttpMonitorStatus::Unknown | HttpMonitorStatus::Inactive, false) => {
            (1, HttpMonitorStatus::Down)
        }

        // Up monitor staying up
        (HttpMonitorStatus::Up, true) => (
            current_status_counter.saturating_add(1),
            HttpMonitorStatus::Up,
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::entities::http_monitor::HttpMonitorStatus;

    use super::next_status;

    #[test]
    fn next_status_tests_with_confirmation() {
        // Transition from unknown to up
        assert_eq!(
            next_status(2, 2, HttpMonitorStatus::Unknown, 0, true),
            (1, HttpMonitorStatus::Up)
        );
        // Transition from unknown to suspicious
        assert_eq!(
            next_status(2, 2, HttpMonitorStatus::Unknown, 0, false),
            (1, HttpMonitorStatus::Suspicious)
        );

        // Down counter increment
        assert_eq!(
            next_status(2, 2, HttpMonitorStatus::Down, 2, false),
            (3, HttpMonitorStatus::Down)
        );
        assert_eq!(
            next_status(2, 2, HttpMonitorStatus::Down, 3, false),
            (4, HttpMonitorStatus::Down)
        );

        // Transition from down to recovering
        assert_eq!(
            next_status(2, 2, HttpMonitorStatus::Down, 3, true),
            (1, HttpMonitorStatus::Recovering)
        );

        // Recovering counter increment (confirmation threshold = 2)
        assert_eq!(
            next_status(2, 2, HttpMonitorStatus::Recovering, 0, true),
            (1, HttpMonitorStatus::Recovering)
        );

        // Transition from recovering to up
        assert_eq!(
            next_status(2, 2, HttpMonitorStatus::Recovering, 2, true),
            (1, HttpMonitorStatus::Up)
        );

        // Up counter increment
        assert_eq!(
            next_status(2, 2, HttpMonitorStatus::Up, 2, true),
            (3, HttpMonitorStatus::Up)
        );
        assert_eq!(
            next_status(2, 2, HttpMonitorStatus::Up, 3, true),
            (4, HttpMonitorStatus::Up)
        );

        // Transition from up to suspicious
        assert_eq!(
            next_status(2, 2, HttpMonitorStatus::Up, 3, false),
            (1, HttpMonitorStatus::Suspicious)
        );

        // Suspicious counter increment
        assert_eq!(
            next_status(2, 2, HttpMonitorStatus::Suspicious, 0, false),
            (1, HttpMonitorStatus::Suspicious)
        );

        // Transition from suspicious to down
        assert_eq!(
            next_status(1, 1, HttpMonitorStatus::Suspicious, 1, false),
            (1, HttpMonitorStatus::Down)
        );
    }

    #[test]
    fn next_status_tests_no_confirmation() {
        // Transition from unknown to up or down
        assert_eq!(
            next_status(1, 1, HttpMonitorStatus::Unknown, 0, true),
            (1, HttpMonitorStatus::Up)
        );
        assert_eq!(
            next_status(1, 1, HttpMonitorStatus::Unknown, 0, false),
            (1, HttpMonitorStatus::Down)
        );

        // Down counter increment
        assert_eq!(
            next_status(1, 1, HttpMonitorStatus::Down, 2, false),
            (3, HttpMonitorStatus::Down)
        );
        assert_eq!(
            next_status(1, 1, HttpMonitorStatus::Down, 3, false),
            (4, HttpMonitorStatus::Down)
        );

        // Transition from down to up (no confirmation)
        assert_eq!(
            next_status(1, 1, HttpMonitorStatus::Down, 3, true),
            (1, HttpMonitorStatus::Up)
        );

        // Up counter increment
        assert_eq!(
            next_status(1, 1, HttpMonitorStatus::Up, 2, true),
            (3, HttpMonitorStatus::Up)
        );
        assert_eq!(
            next_status(1, 1, HttpMonitorStatus::Up, 3, true),
            (4, HttpMonitorStatus::Up)
        );

        // Transition from up to down
        assert_eq!(
            next_status(1, 1, HttpMonitorStatus::Up, 3, false),
            (1, HttpMonitorStatus::Down)
        );

        // Transition from suspicious to down
        assert_eq!(
            next_status(1, 1, HttpMonitorStatus::Suspicious, 0, false),
            (1, HttpMonitorStatus::Down)
        );

        // Transition from recovering to up
        assert_eq!(
            next_status(1, 1, HttpMonitorStatus::Recovering, 0, true),
            (1, HttpMonitorStatus::Up)
        );
    }
}
