# trmnl-server

This is a TRMNL BYOS server implementation written in rust. This is currently in active development.

## Limitations

Currently the following features are missing:

- No way to change the refresh rate
- No way to update the firmware OTA
- No support for user authentication and device management permissions

## Endpoints

### `GET /api/setup`

Called by device to setup and exchange API key

### `GET /api/display`

Called by device to request a new image for display

### `GET /api/log`

> TODO these should be persisted
> Called by device to share logs

### `GET /api/devices`

Management endpoint to retrieve a list of devices and their information

#### Example response

```json
[
  {
    "id": "57D415",
    "mac": "28:37:2F:AA:15:88",
    "rssi": -69,
    "battery_voltage": 3.88,
    "fw_version": "1.6.5",
    "refresh_rate": 900
  }
]
```

### `GET /api/devices/<DEVICE_ID>`

Management endpoint to retrieve device information

#### Example response

```json
{
  "id": "57D415",
  "mac": "28:37:2F:AA:15:88",
  "rssi": -69,
  "battery_voltage": 3.88,
  "fw_version": "1.6.5",
  "refresh_rate": 900
}
```

### `GET /api/devices/<DEVICE_ID>/images`

Management endpoint to get the current images on rotation for a device

#### Example response

```json
["https://url_to_image_1.png", "https://url_to_image_2.png"]
```

### `PUT /api/devices/<DEVICE_ID>/images`

Management endpoint to update the current images on rotation for a device

#### Example request

```json
["https://url_to_image_1.png", "https://url_to_image_2.png"]
```

#### Example response

```json
["https://url_to_image_1.png", "https://url_to_image_2.png"]
```

## Local development

### Adding a migration

```sh
sqlx migrate add NAME_OF_MIGRATION
```

### Running migrations

```sh
sqlx migrate run
```

### Prepare queries

```sh
cargo sqlx prepare
```
