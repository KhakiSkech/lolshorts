/// Send a privacy-minimal operational signal.
///
/// Only compile-time categories and error codes are accepted. File paths, user
/// identifiers, OAuth data, command arguments, and error messages are never
/// attached to the event.
pub fn capture_operational_error(category: &'static str, error_code: &'static str) {
    sentry::with_scope(
        |scope| {
            scope.set_tag("lolshorts.category", category);
            scope.set_tag("lolshorts.error_code", error_code);
        },
        || {
            sentry::capture_message("lolshorts_operational_error", sentry::Level::Error);
        },
    );
}
