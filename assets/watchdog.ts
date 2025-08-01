/// <reference lib="dom" />
function nop() { }

const document_create_element = document?.createElement?.bind(document) ?? nop;
const set_timeout = globalThis?.setTimeout ?? nop;

const console_debug = console?.debug?.bind(console) ?? nop;
const console_log = console?.log?.bind(console) ?? nop;
const console_warn = console?.warn?.bind(console) ?? nop;
const console_error = console?.error?.bind(console) ?? nop;

type State = {
    last_heartbeat: number,
    heartbeat_interval_ms: number | null,
    watchdog_timeout_ms: number | null,
    watchdog_beat_misses: number,
    timeout_id: number | null,
    tabbed_out: boolean,
};

const state = {
    last_heartbeat: Date.now(),
    heartbeat_interval_ms: null,
    watchdog_timeout_ms: null,
    watchdog_beat_misses: 0,
    timeout_id: null,
    tabbed_out: false,
} as State;

const WATCHDOG_MISS_THRESHOLDS = {
    WARN: 2,
    ALERT: 10,
    DIALOG: 30,
} as const;

/**
 * Initializes the watchdog with the specified interval and timeout.
 */
function init_watchdog(heartbeat_interval_ms: number, watchdog_timeout_ms: number)
{
    state.last_heartbeat = Date.now();
    state.heartbeat_interval_ms = heartbeat_interval_ms;
    state.watchdog_timeout_ms = watchdog_timeout_ms;

    set_timeout(watchdog_loop, watchdog_timeout_ms);

    console_log(`Watchdog initialized with interval: ${heartbeat_interval_ms} ms, timeout: ${watchdog_timeout_ms} ms`);
}

function heartbeat()
{
    state.last_heartbeat = Date.now();
    // console_debug("Heartbeat");
}

(globalThis as { [key: string]: unknown }).init_watchdog = init_watchdog;
(globalThis as { [key: string]: unknown }).heartbeat = heartbeat;

function watchdog_loop()
{
    state.timeout_id = null;
    const last_heartbeat = state.last_heartbeat;
    const now = Date.now();
    const timeout = state.watchdog_timeout_ms;

    if (timeout == null)
    {
        return;
    }

    if (last_heartbeat + timeout < now && !state.tabbed_out)
    {
        ++state.watchdog_beat_misses;
        on_beat_miss(state.watchdog_beat_misses, timeout);
    } else
    {
        if (state.watchdog_beat_misses >= WATCHDOG_MISS_THRESHOLDS.WARN)
        {
            console_log(`Heartbeat restored after ${state.watchdog_beat_misses * timeout} ms`);
        }

        state.watchdog_beat_misses = 0;
    }

    const timeout_id = set_timeout(watchdog_loop, timeout);
    state.timeout_id = timeout_id;
}

function on_beat_miss(misses: number, timeout: number)
{
    if (misses == WATCHDOG_MISS_THRESHOLDS.WARN)
    {
        console_warn(`No heartbeats in over ${misses * timeout} ms. Is the WASM engine busy?`);
    } else if (misses == WATCHDOG_MISS_THRESHOLDS.ALERT)
    {
        console_warn(`The WASM engine seems unresponsive (no heartbeats in ${misses * timeout} ms).`);
    } else if (misses == WATCHDOG_MISS_THRESHOLDS.DIALOG)
    {
        console_error(`No heartbeats from WASM engine after ${misses * timeout} ms. Displaying frozen message.`);
        try
        {
            watchdog_dialog(misses * timeout);
        } catch (e)
        {
            console_error(`Error trying to display panic message: ${e}`);
        }
    }
}

function watchdog_dialog(total_time: number)
{
    const dialog = document_create_element("dialog");
    dialog.setAttribute("open", "");

    if (typeof dialog == "undefined" || dialog == null)
    {
        throw new Error("Could not create dialog element");
    }

    document?.body?.appendChild(dialog);

    try
    {
        const h1 = document_create_element("h1");
        h1.textContent = "Frozen!";
        dialog.appendChild(h1);
    } catch
    {
        // We don't care about failures here
    }

    const p = document_create_element("p");
    p.textContent = `No heartbeats from WASM engine after ${total_time} ms. Check the console for any possible errors.`;
    dialog.appendChild(p);

    try
    {
        const button = document_create_element("button");
        button.textContent = "Dismiss";
        button.setAttribute("onclick", "this.parentElement.close()");
        dialog.appendChild(button);
    } catch
    {
        // We don't care about failures here
    }
}

function handle_visibility_change()
{
    if (document?.visibilityState === "hidden")
    {
        state.tabbed_out = true;
    } else if (document?.visibilityState === "visible")
    {
        state.tabbed_out = false;
    }
}

document?.addEventListener("visibilitychange", handle_visibility_change);