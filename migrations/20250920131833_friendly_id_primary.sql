-- Rename the existing table
ALTER TABLE devices RENAME TO devices_old;

-- Create the new table with NOT NULL and UNIQUE constraints
CREATE TABLE devices (
    id TEXT NOT NULL PRIMARY KEY,
    mac TEXT,
    api_key TEXT NOT NULL UNIQUE,
    rssi INTEGER,
    battery_voltage REAL,
    fw_version TEXT,
    refresh_rate INTEGER
);

-- Copy existing data
INSERT INTO devices (id, api_key, mac, rssi, battery_voltage, fw_version, refresh_rate)
SELECT friendly_id, api_key, mac, rssi, battery_voltage, fw_version, refresh_rate
FROM devices_old;

-- Drop the old table
DROP TABLE devices_old;
