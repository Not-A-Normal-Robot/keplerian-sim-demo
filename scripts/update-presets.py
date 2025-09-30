#!/usr/bin/env python3
from dataclasses import dataclass
from pathlib import Path
import requests
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry
from urllib.parse import urlencode
import tempfile
import shutil

@dataclass
class Elements:
    """Describes the elements returned by the API call in its original form."""

    ec: float
    """Eccentricity, dimensionless"""
    qr: float
    """Periapsis radius, in km"""
    in_: float
    """Inclination w.r.t. ecliptic, in degrees"""
    om: float
    """Longitude of ascending node w.r.t. ecliptic, in degrees"""
    w: float
    """Argument of periapsis w.r.t. ecliptic, in degrees"""
    ma: float
    """Mean anomaly, in degrees"""

TOML_REL_PATH = Path("../src/sim/presets.toml")  # Relative to script dir root
TDB_TIMESTAMP = 2460946.166666700 # A.D. 2025-Sep-27 16:00:00.0029 TDB 
API_BASE = "https://ssd.jpl.nasa.gov/api/horizons.api"
API_BASE_PARAMS = {
    "format": "text",
    "OBJ_DATA": "NO",
    "EPHEM_TYPE": "ELEMENTS",
    "MAKE_EPHEM": "YES",
    "REF_PLANE": "ECLIPTIC",
    "TIME_TYPE": "TDB",
    "TLIST": f"'{TDB_TIMESTAMP:.9f}'",
    "TLIST_TYPE": "JD",
    "ELM_LABELS": "NO",
}
BODY_MAP: dict[str, tuple[str, str] | None] = {
    # Maps between the TOML name, and the
    # tuple containing the Horizons ID and its parent ID.

    # STARS
    "the_sun": None,

    # MAIN PLANETS
    "mercury": ("199", "10"),
    "venus": ("299", "10"),
    "earth": ("399", "10"),
    "mars": ("499", "10"),
    "jupiter": ("599", "10"),
    "saturn": ("699", "10"),
    "uranus": ("799", "10"),
    "neptune": ("899", "10"),

    # MOONS
    "luna": ("301", "399"),
    "phobos": ("401", "499"),
    "deimos": ("402", "499"),
    "io": ("501", "599"),
    "europa": ("502", "599"),
    "ganymede": ("503", "599"),
    "callisto": ("504", "599"),
    "titan": ("606", "699"),
    "enceladus": ("602", "699"),
    "mimas": ("601", "699"),
    "tethys": ("603", "699"),
    "iapetus": ("608", "699"),
    "titania": ("703", "799"),
    "oberon": ("704", "799"),
    "triton": ("801", "899"),
    "proteus": ("808", "899"),
    "nereid": ("802", "899"),
    "weywot": ("120050000", "920050000"),  # Quaoar I, parent Quaoar
    "charon": ("901", "999"), # parent Pluto
    "dysnomia": ("120136199", "920136199"), # parent Eris

    # MINOR PLANETS (dwarf planets, asteroids, TNOs)
    "ceres": ("1;", "10"),
    "vesta": ("4;", "10"),
    "quaoar": ("50000;", "10"),
    "sedna": ("90377;", "10"),
    "leleakuhonua": ("541132;", "10"),
    "pluto": ("134340;", "10"),
    "haumea": ("136108;", "10"),
    "eris": ("136199;", "10"),
    "makemake": ("136472;", "10"),

    # ARTIFICIAL SATELLITES
    "parker_solar_probe": ("-96", "10"),
    "voyager_1": ("-31", "10"),
    "voyager_2": ("-32", "10"),
    "new_horizons": ("-98", "10"),
    "pioneer_10": ("-23", "10"),
    "pioneer_11": ("-24", "10"),
    "geostationary_sat": None,
}

remaining_bodies = set(BODY_MAP.keys())

SESSION = requests.Session()
SESSION.mount(
    "https://",
    HTTPAdapter(
        max_retries=Retry(
            total=3,
            status_forcelist=[429, 500, 502, 503, 504],
            allowed_methods=["GET"],
            backoff_factor=1,
        )
    )
)

# Default timeout (in seconds) for requests; used as a per-request timeout.
DEFAULT_TIMEOUT = 10

def get_api_params(name: str) -> dict[str, str] | None:
    if name not in BODY_MAP:
        raise Exception(f"Body '{name}' is not in BODY_MAP (unrecognized body)")
    value = BODY_MAP[name]
    if value is None:
        return None
    this, parent = value

    command = f"'{this}'"
    center = f"@{parent}"

    params = API_BASE_PARAMS.copy()
    params.update({
        "COMMAND": command,
        "CENTER": center,
    })
    return params

def get_api_url(params: dict[str, str]) -> str:
    return f"{API_BASE}?{urlencode(params)}"

def parse_ephemeris(ephemeris_text: str) -> Elements | None:
    ELEMENTS_START_TEXT = "$$SOE"
    ELEMENTS_END_TEXT = "$$EOE"
    start_idx = ephemeris_text.find(ELEMENTS_START_TEXT)
    end_idx = ephemeris_text.find(ELEMENTS_END_TEXT)

    if start_idx == -1 or end_idx == -1:
        return None
    
    data_text = ephemeris_text[
        start_idx + len(ELEMENTS_START_TEXT):end_idx
    ].strip()
    
    # Skip the timestamp after $$SOE
    data_lines = data_text.splitlines()
    if len(data_lines) < 1:
        return None
    elements_wsv = " ".join(data_lines[1:]).strip()
    elements_arr = [float(el) for el in elements_wsv.split()]
    return Elements(
        ec = elements_arr[0],
        qr = elements_arr[1],
        in_ = elements_arr[2],
        om = elements_arr[3],
        w = elements_arr[4],
        ma = elements_arr[7],
    )

def get_elements(name: str) -> Elements | None:
    params = get_api_params(name)
    if params is None:
        return None
    
    url = get_api_url(params)
    try:
        response = SESSION.get(url, timeout=DEFAULT_TIMEOUT)
    except Exception as e:
        raise e
    response.raise_for_status()
    elements = parse_ephemeris(response.text)
    if elements is None:
        raise Exception(
            "Could not parse ephemeris. Could this be an API error?\n" +
            "Server response:\n" +
            response.text
        )
    return elements
    
def get_toml_file_path() -> str:
    return str((Path(__file__).resolve().parent / TOML_REL_PATH).resolve())

def todo():
    raise Exception("not implemented yet")

if __name__ == "__main__":
    modified_lines: dict[int, str] = dict()
    toml_path = get_toml_file_path()

    current_body: str | None = None
    current_elements: Elements | None = None
    with open(toml_path, "r") as f:
        for line_num, line in enumerate(f):
            line = line.strip()
            if line.startswith("[") and line.endswith("]"):
                current_body = line[1:-1]
                remaining_bodies.remove(current_body)
                current_elements = None
                try:
                    print(f"Fetching API for {current_body}...")
                    current_elements = get_elements(current_body)
                except Exception as exception:
                    print(
                        f"Error occurred fetching API for data about {current_body}",
                        exception
                    )
            if current_elements is None:
                continue
            new_line: str | None = None
            if line.startswith("apoapsis") or line.startswith("eccentricity"):
                new_line = f"eccentricity = {current_elements.ec}"
            if line.startswith("periapsis"):
                new_line = f"periapsis = {current_elements.qr * 1e3}"
            if line.startswith("inclination"):
                new_line = f"inclination = {current_elements.in_}"
            if line.startswith("arg_pe"):
                new_line = f"arg_pe = {current_elements.w}"
            if line.startswith("long_asc_node"):
                new_line = f"long_asc_node = {current_elements.om}"
            if line.startswith("mean_anomaly"):
                new_line = f"mean_anomaly = {current_elements.ma}"

            if new_line is not None:
                modified_lines[line_num] = new_line

    if remaining_bodies:
        print("ERROR: The following bodies are in BODY_MAP but not present in the TOML file (desynchronization):")
        for body in sorted(remaining_bodies):
            print(f"  - {body}")

    with open(toml_path, "r") as src, tempfile.NamedTemporaryFile("w", delete=False) as tmp:
        for i, line in enumerate(src):
            if i in modified_lines:
                tmp.write(modified_lines[i] + "\n")
            else:
                tmp.write(line)
    shutil.move(tmp.name, toml_path)
        