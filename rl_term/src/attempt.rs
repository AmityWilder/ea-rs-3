use ext_trait::extension;
use std::fmt::Display;

#[macro_export]
macro_rules! attempt {
    ($($args:tt)+) => {
        $crate::logln!(Attempt, "{}...", format_args!($($args)+))
    };
}

#[extension(pub trait AttemptResult)]
impl<T, E> Result<T, E> {
    /// If `self` is [`Ok`], print `pass_msg` to the global logger as [`super::LogType::Info`]
    #[inline]
    fn ok_info<D>(self, pass_msg: D) -> Self
    where
        D: Display,
    {
        if self.is_ok() {
            logln!(Info, "{pass_msg}.");
        }

        self
    }

    /// If `self` is [`Ok`], print `pass_msg` to the global logger as [`super::LogType::Info`]
    #[inline]
    fn ok_info_with<D, F>(self, pass_msg: F) -> Self
    where
        D: Display,
        F: FnOnce(&T) -> D,
    {
        self.inspect(|v| logln!(Info, "{}.", pass_msg(v)))
    }

    /// If `self` is [`Err`], print `pass_msg` to the global logger as [`super::LogType::Info`]
    #[inline]
    fn err_info<D>(self, fail_msg: D) -> Self
    where
        D: Display,
    {
        if self.is_err() {
            logln!(Info, "{fail_msg}.");
        }

        self
    }

    /// If `self` is [`Err`], print `pass_msg` to the global logger as [`super::LogType::Info`]
    #[inline]
    fn err_info_with<D, F>(self, fail_msg: F) -> Self
    where
        D: Display,
        F: FnOnce(&E) -> D,
    {
        self.inspect_err(|e| logln!(Info, "{}.", fail_msg(e)))
    }

    /// Print `msg` to the global logger as [`super::LogType::Info`], regardless of `self`
    #[inline]
    fn info<D>(self, msg: D) -> Self
    where
        D: Display,
    {
        logln!(Info, "{msg}.");

        self
    }

    /// Print `msg` to the global logger as [`super::LogType::Info`], regardless of `self`
    #[inline]
    fn info_with<D, F>(self, msg: F) -> Self
    where
        D: Display,
        F: FnOnce(&Self) -> D,
    {
        logln!(Info, "{}.", msg(&self));

        self
    }

    /// If `self` is [`Ok`], print `pass_msg` to the global logger as a [`super::LogType::Success`]
    #[inline]
    fn success<D>(self, pass_msg: D) -> Self
    where
        D: Display,
    {
        if self.is_ok() {
            logln!(Success, "{pass_msg}.");
        }

        self
    }

    /// If `self` is [`Ok`], print `pass_msg` to the global logger as a [`super::LogType::Success`]
    #[inline]
    fn success_with<D, F>(self, pass_msg: F) -> Self
    where
        D: Display,
        F: FnOnce(&T) -> D,
    {
        self.inspect(|v| logln!(Success, "{}.", pass_msg(v)))
    }

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as a [`super::LogType::Warning`]
    ///
    /// Appends error to log
    #[inline]
    fn warn<D>(self, fail_msg: D) -> Self
    where
        E: Display,
        D: Display,
    {
        self.inspect_err(|e| logln!(Warning, "{fail_msg}: {e}"))
    }

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as a [`super::LogType::Warning`]
    ///
    /// Appends error to log
    #[inline]
    fn warn_with<D, F>(self, fail_msg: F) -> Self
    where
        E: Display,
        D: Display,
        F: FnOnce(&E) -> D,
    {
        self.inspect_err(|e| logln!(Warning, "{}: {e}", fail_msg(e)))
    }

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as a [`super::LogType::Warning`], consuming `self`
    ///
    /// Appends error to log
    #[inline]
    fn issue_warning<D>(self, fail_msg: D)
    where
        E: Display,
        D: Display,
    {
        if let Err(e) = self {
            logln!(Warning, "{fail_msg}: {e}");
        }
    }

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as a [`super::LogType::Warning`], consuming `self`
    ///
    /// Appends error to log
    #[inline]
    fn issue_warning_with<D, F>(self, fail_msg: F)
    where
        E: Display,
        D: Display,
        F: FnOnce(&E) -> D,
    {
        if let Err(e) = self {
            logln!(Warning, "{}: {e}", fail_msg(&e));
        }
    }

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as a [`super::LogType::Warning`]
    ///
    /// Appends error to log
    ///
    /// Returns `self` if [`Ok`], or [`Default::default()`] if [`Err`]
    #[inline]
    fn warn_or_default<D>(self, fail_msg: D) -> T
    where
        T: Default,
        E: Display,
        D: Display,
    {
        self.inspect_err(|e| logln!(Warning, "{fail_msg}: {e}"))
            .unwrap_or_default()
    }

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as a [`super::LogType::Warning`]
    ///
    /// Appends error to log
    ///
    /// Returns `self` if [`Ok`], or [`Default::default()`] if [`Err`]
    #[inline]
    fn warn_with_or_default<D, F>(self, fail_msg: F) -> T
    where
        T: Default,
        E: Display,
        D: Display,
        F: FnOnce(&E) -> D,
    {
        self.inspect_err(|e| logln!(Warning, "{}: {e}", fail_msg(e)))
            .unwrap_or_default()
    }

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as a [`super::LogType::Warning`]
    ///
    /// Appends error to log
    ///
    /// Returns `self` if [`Ok`], or `default` if [`Err`]
    #[inline]
    fn warn_or<D>(self, fail_msg: D, default: T) -> T
    where
        E: Display,
        D: Display,
    {
        self.inspect_err(|e| logln!(Warning, "{fail_msg}: {e}"))
            .unwrap_or(default)
    }

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as a [`super::LogType::Warning`]
    ///
    /// Appends error to log
    ///
    /// Returns `self` if [`Ok`], or `default` if [`Err`]
    #[inline]
    fn warn_with_or<D, F>(self, fail_msg: F, default: T) -> T
    where
        E: Display,
        D: Display,
        F: FnOnce(&E) -> D,
    {
        self.inspect_err(|e| logln!(Warning, "{}: {e}", fail_msg(e)))
            .unwrap_or(default)
    }

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as a [`super::LogType::Warning`]
    ///
    /// Appends error to log
    ///
    /// Returns `self` if [`Ok`], or evaluates and returns `default` if [`Err`]
    #[inline]
    fn warn_or_else<D, F, G>(self, fail_msg: D, default: G) -> T
    where
        E: Display,
        D: Display,
        G: FnOnce(E) -> T,
    {
        self.inspect_err(|e| logln!(Warning, "{fail_msg}: {e}"))
            .unwrap_or_else(default)
    }

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as a [`super::LogType::Warning`]
    ///
    /// Appends error to log
    ///
    /// Returns `self` if [`Ok`], or evaluates and returns `default` if [`Err`]
    #[inline]
    fn warn_with_or_else<D, F, G>(self, fail_msg: F, default: G) -> T
    where
        E: Display,
        D: Display,
        F: FnOnce(&E) -> D,
        G: FnOnce(E) -> T,
    {
        self.inspect_err(|e| logln!(Warning, "{}: {e}", fail_msg(e)))
            .unwrap_or_else(default)
    }

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as an [`super::LogType::Error`]
    ///
    /// Appends error to log
    #[inline]
    fn error<D>(self, fail_msg: D) -> Self
    where
        E: Display,
        D: Display,
    {
        self.inspect_err(|e| logln!(Error, "{fail_msg}: {e}"))
    }

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as an [`super::LogType::Error`]
    ///
    /// Appends error to log
    #[inline]
    fn error_with<D, F>(self, fail_msg: F) -> Self
    where
        E: Display,
        D: Display,
        F: FnOnce(&E) -> D,
    {
        self.inspect_err(|e| logln!(Error, "{}: {e}", fail_msg(e)))
    }

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as an [`super::LogType::Error`]
    ///
    /// Appends error to log
    #[inline]
    fn error_unwrap(self) -> Self
    where
        E: Display,
    {
        self.inspect_err(|e| logln!(Error, "{e}"))
    }

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as an [`super::LogType::Error`], consuming `self`
    ///
    /// Appends error to log
    #[inline]
    fn issue_error<D>(self, fail_msg: D)
    where
        E: Display,
        D: Display,
    {
        if let Err(e) = self {
            logln!(Error, "{fail_msg}: {e}");
        }
    }

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as an [`super::LogType::Error`], consuming `self`
    ///
    /// Appends error to log
    #[inline]
    fn issue_error_with<D, F>(self, fail_msg: F)
    where
        E: Display,
        D: Display,
        F: FnOnce(&E) -> D,
    {
        if let Err(e) = self {
            logln!(Error, "{}: {e}", fail_msg(&e));
        }
    }

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as [`super::LogType::Fatal`] and panic
    ///
    /// Appends error to log
    #[inline]
    fn fatal<D>(self, fail_msg: D) -> T
    where
        E: Display,
        D: Display,
    {
        self.unwrap_or_else(|e| {
            logln!(Error, "{fail_msg}: {e}");
            panic!("fatal error: {fail_msg}: {e}");
        })
    }

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as [`super::LogType::Fatal`] and panic
    ///
    /// Appends error to log
    #[inline]
    fn fatal_with<D, F>(self, fail_msg: F) -> T
    where
        E: Display,
        D: Display,
        F: FnOnce(&E) -> D,
    {
        self.unwrap_or_else(|e| {
            let msg = fail_msg(&e);
            logln!(Error, "{msg}: {e}");
            panic!("fatal error: {msg}: {e}");
        })
    }

    /// If `self` is [`Err`], print `fail_msg` and the error to the global logger
    /// as [`super::LogType::Fatal`] and panic
    ///
    /// Appends error to log
    #[inline]
    fn fatal_unwrap(self) -> T
    where
        E: Display,
    {
        self.unwrap_or_else(|e| {
            logln!(Error, "{e}");
            panic!("fatal error: {e}");
        })
    }
}
