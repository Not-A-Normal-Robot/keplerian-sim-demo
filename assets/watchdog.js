const document_createElement = document.createElement.bind(document);

const console_log = console.log.bind(console);
const console_warn = console.warn.bind(console);
const console_error = console.error.bind(console);

let last_heartbeat = Date.now();

/** @type {number | null} */
let heartbeat_interval_ms = null;

/** @type {number | null} */
let watchdog_timeout_ms = null;

/**
 * Initializes the watchdog with the specified interval and timeout.
 * @param {number} _heartbeat_interval_ms - The interval in milliseconds at which the watchdog should check.
 * @param {number} _watchdog_timeout_ms - The timeout in milliseconds after which the watchdog should trigger an alert if no activity is detected.
 */
export function init_watchdog(_heartbeat_interval_ms, _watchdog_timeout_ms)
{
    if (typeof _heartbeat_interval_ms !== 'number' || typeof _watchdog_timeout_ms !== 'number')
    {
        console_error('Invalid parameters for init_watchdog');
        return;
    }

    if (_heartbeat_interval_ms <= 0)
    {
        console_error('watchdog_interval_ms must be greater than 0');
        return;
    }

    if (_watchdog_timeout_ms <= 0)
    {
        console_error('watchdog_timeout_ms must be greater than 0');
        return;
    }

    heartbeat_interval_ms = _heartbeat_interval_ms;
    watchdog_timeout_ms = _watchdog_timeout_ms;
    console_log(`Watchdog initialized with interval: ${heartbeat_interval_ms} ms, timeout: ${watchdog_timeout_ms} ms`);
}

function watchdog_loop()
{
    if (heartbeat_interval_ms === null || watchdog_timeout_ms === null)
    {
        console_warn('Watchdog not initialized. Call init_watchdog first.');
        return;
    }

    const now = Date.now();
    if (now - last_heartbeat > watchdog_timeout_ms)
    {
        console_error('Watchdog timeout exceeded! No heartbeat received in the last ' + watchdog_timeout_ms + ' ms');
        alert('Watchdog timeout exceeded! The application may be unresponsive.');
        return;
    }

    setTimeout(watchdog_loop, heartbeat_interval_ms);
}