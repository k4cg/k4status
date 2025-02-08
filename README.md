# Space API for K4CG

The software serves the Space API JSON document upon a HTTP request.
The document itself is provided as a template to which the latest sensor values will be added.
The sensor values are collected from an InfluxDB that will be continiously updated from HomeAssistant.

[![Door status](https://spaceapi.k4cg.org/badge)](https://k4cg.org)


## Endpoints

- GET `/status.json`: Request the current space status compliant to the Space API JSON schema.
- GET `/health`: Get health information, basically checks the connection to the database. Returns 200 if everything is ok, and 418 in case the connection could not be established.
- GET `/badge`: Get the current door status as a badge


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
            "entity": "", // Entity ID of sensor within database
            "unit": "state", // Unit of measurement within database
            "validity": 0 // Period of validity in seconds, set to 0 to disable check
        },
        "temperature": {
            "id": [ // List of sensors
                {
                    "entity": "", // Entity ID of sensor within database
                    "location": "" //  Human readable location of sensor
                }
            ],
            "unit": "°C", // Unit of measurements within database
            "validity": 0 // Period of validity in seconds, set to 0 to disable check
        },
        "humidity": {
            "id": [ // List of sensors
                {
                    "entity": "", // Entity ID of sensor within database
                    "location": "" // Human readable location of sensor
                }
            ],
            "unit": "%", // Unit of measurement within database
            "validity": 0 // Period of validity in seconds, set to 0 to disable check
        },
        "carbondioxide": {
            "id": [ // List of sensors
                {
                    "entity": "", // Entity ID of sensor within database
                    "location": "" // Human readable location of sensor
                }
            ],
            "unit": "ppm", // Unit of measurement within database
            "validity": 0 // Period of validity in seconds, set to 0 to disable check
        }
    },
    "cache_time": {
        "status.json": 0, // Time in seconds to cache values for /status.json endpoint, set to 0 to disable caching
        "health": 0, // Time in seconds to cache values for /health endpoint, set to 0 to disable caching
        "badge": 0 // Time in seconds to cache values for /badge endpoint, set to 0 to disable caching
    }
}
```

The template for the Space API JSON document is defined within the file `template.json`.
You may have a look in the Space API specification on how to fill the fields.


## Run

By default, the app will look for both the configuration (`config.json`) and template (`template.json`) files in the current working directory.
The badges are expected to be located in the folder `badges/` with the names `open.svg`, `closed.svg` and `unknown.svg`.
In case the files are located somewhere different, you can use the optional commandline arguments or their corresponding environment variables to provide the correct paths.
Just run `./k4status --help` to show their usage.

Next to the file paths, you can adjust the log level.
You may use the environment variable `RUST_LOG` to configure anything other than the default level `info`.


## Build

Building the software requires a Rust toolchain to be installed.
If so, the build process is as easy as `cargo build [--release]`.

Two minimalistic tests are provided to check wether the configuration and the template files can be parsed.
Run `cargo test` to execute the tests.
