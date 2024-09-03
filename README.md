# Space API for K4CG

The software serves the Space API JSON document upon a HTTP request.
The document itself is provided as a template to which the latest sensor values will be added.
The sensor values are collected from an InfluxDB that will be continiously updated from HomeAssistant.


## Endpoints

- GET `/status.json`: Request the current space status compliant to the Space API JSON schema.
- GET `/health`: Get health information, basically checks the connection to the database. Returns 200 if everything is ok, and 418 in case the connection could not be established.


## Configuration

The configuration takes place within the file `config.json`:

```json5
{
    "database": {
        "connection": "", // Connection string, e.g. 'http://localhost:8086'
        "database": "", // Name of database
        "username": "", // Username for auth
        "password": "" // Password for auth
    },
    "server": {
        "hostname": "localhost", // Local IP/hostname to use for serving data
        "port": 3000 // Local port to use for serving data
    },
    "sensors": {
        "door": {
            "name": "", // Name/ID of sensor within database
            "unit": "state", // Unit of measurement within database
            "validity": 0 // Period of validity in seconds, set to 0 to disable check
        },
        "temperature": {
            "name": [ // List of sensors
                {
                    "id": "", // Name/ID of sensor within database
                    "location": "" // Location of sensor
                }
            ],
            "unit": "°C", // Unit of measurements within database
            "validity": 0 // Period of validity in seconds, set to 0 to disable check
        },
        "humidity": {
            "name": [ // List of sensors
                {
                    "id": "", // Name/ID of sensor within database
                    "location": "" // Location of sensor
                }
            ],
            "unit": "%", // Unit of measurement within database
            "validity": 0 // Period of validity in seconds, set to 0 to disable check
        }
    },
    "cache_time": {
        "status.json": 0, // Time in seconds to cache values for /status.json endpoint, set to 0 to disable caching
        "health": 0 // Time in seconds to cache values for /health endpoint, set to 0 to disable caching
    }
}
```

The template for the Space API JSON document is defined within the file `status.json`.


## Build

Building the software requires a Rust toolchain to be installed.
If so, the build process is as easy as `cargo build [--release]`.
Currently, the build defaults to x64 with musl on Linux (see `.cargo/config.toml`).
You may override it by providing the necessary arguments to cargo.

Two minimalistic tests are provided to check wether the configuration and the template files can be parsed.
Run `cargo test` to execute the tests.
