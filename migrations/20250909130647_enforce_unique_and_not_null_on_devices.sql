-- Rename the existing table
ALTER TABLE devices RENAME TO devices_old;

-- Create the new table with NOT NULL and UNIQUE constraints
CREATE TABLE devices (
    mac TEXT NOT NULL PRIMARY KEY,
    api_key TEXT NOT NULL UNIQUE,
    friendly_id TEXT NOT NULL UNIQUE,
    rssi INTEGER,
    battery_voltage REAL,
    fw_version TEXT,
    refresh_rate INTEGER
);

-- Copy existing data
INSERT INTO devices (mac, api_key, friendly_id, rssi, battery_voltage, fw_version, refresh_rate)
SELECT mac, api_key, friendly_id, rssi, battery_voltage, fw_version, refresh_rate
FROM devices_old;

-- Drop the old table
DROP TABLE devices_old;
