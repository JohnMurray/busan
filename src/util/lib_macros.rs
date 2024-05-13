//! A collection of handy macros for internal use in Busan.

macro_rules! channel_send {
    ($e:expr, $ee:expr) => {{
        let res = ($e).send($ee);
        // Provide a more helpeful error message during debug runs
        debug_assert!(
            res.is_ok(),
            "Unable to send message across channel: {}",
            res.as_ref().err().unwrap()
        );
        // In production, emit an error log, but do not panic
        if res.is_err() {
            ::log::error!(
                "Unable to send message across channel, {}",
                res.err().unwrap()
            );
        }
    }};
}

macro_rules! channel_must_recv {
    ($e:expr) => {{
        let res = ($e).recv();
        // Emit a useful error log
        if res.is_err() {
            ::log::error!(
                "Unable to receive message from channel, {}",
                res.as_ref().err().unwrap()
            );
        }
        // Unwrap the result, possibly panic
        res.unwrap()
    }};
}

pub(crate) use channel_must_recv;
pub(crate) use channel_send;
