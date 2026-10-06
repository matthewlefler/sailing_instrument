#include <Wire.h>
#include <Adafruit_BNO08x.h>

#define BNO_SCL_PIN 4
#define BNO_SDA_PIN 5
#define BNO_INT_PIN 6
#define BNO_RST_PIN 7
#define BNO_DI_PIN 15
#define BNO_CS_PIN 16
#define BNO_SENSOR_ID 101

Adafruit_BNO08x bno08x(BNO_RST_PIN);
sh2_SensorValue_t sensorValue;

typedef struct DeltaTime {
  double dt = 1; // in seconds
  int64_t current = 1;
  int64_t last = 0;
} DeltaTime;

const double MICRO_SEC_TO_SEC = (1.0 / 1000000.0);
void update_deltatime(DeltaTime *dt) {
  dt->current = esp_timer_get_time();
  dt->dt = (double)(dt->current - dt->last) * MICRO_SEC_TO_SEC; // in micro seconds
  dt->last = esp_timer_get_time();
}

DeltaTime gyro_dt;
DeltaTime accel_dt;

void enableReports() {
  // Update interval is in microseconds.
  // 100000 us = 10 Hz.
  if (!bno08x.enableReport(SH2_LINEAR_ACCELERATION, 100000)) {
    Serial.println("Could not enable accelerometer");
  }

  if (!bno08x.enableReport(SH2_GYROSCOPE_CALIBRATED, 100000)) {
    Serial.println("Could not enable gyroscope");
  }

  // if (!bno08x.enableReport(SH2_MAGNETIC_FIELD_CALIBRATED, 100000)) {
  //   Serial.println("Could not enable magnetometer");
  // }
}

void setup() {
  Serial.begin(115200);
  while(!Serial) {
    delay(10);
  }

  Serial.println();
  Serial.println("BNO085 test");

  // .begin(sck, miso, mosi, ss) note ss is the same as the cs pin
  SPI.begin(BNO_SCL_PIN, BNO_SDA_PIN, BNO_DI_PIN, BNO_CS_PIN);

  if (!bno08x.begin_SPI(BNO_CS_PIN, BNO_INT_PIN, &SPI, BNO_SENSOR_ID)) {
    Serial.println("BNO085 not detected.");
    Serial.println("Check pins are connected and the BNO085 has power.");
    while (true) {
      delay(10);
    }
  }

  Serial.println("BNO085 found!");

  enableReports();
  delay(200);
}

double absolute_gyro_x = 0.0;
double absolute_gyro_y = 0.0;
double absolute_gyro_z = 0.0;

double position_x = 0.0;
double position_y = 0.0;
double position_z = 0.0;
void loop() {
  uint8_t count = 0; // used to allow other functions to run other than the bno's
  while (bno08x.getSensorEvent(&sensorValue) && count++ < 10) {
    switch (sensorValue.sensorId) {
      case SH2_LINEAR_ACCELERATION:
        update_deltatime(&accel_dt);
        position_x += sensorValue.un.linearAcceleration.x * accel_dt.dt;
        position_y += sensorValue.un.linearAcceleration.y * accel_dt.dt;
        position_z += sensorValue.un.linearAcceleration.z * accel_dt.dt;
        break;

      case SH2_GYROSCOPE_CALIBRATED:
        update_deltatime(&gyro_dt);
        absolute_gyro_x += sensorValue.un.gyroscope.x * gyro_dt.dt;
        absolute_gyro_y += sensorValue.un.gyroscope.y * gyro_dt.dt;
        absolute_gyro_z += sensorValue.un.gyroscope.z * gyro_dt.dt;
        break;

      // case SH2_MAGNETIC_FIELD_CALIBRATED:
      //   Serial.print("MAG    X: ");
      //   Serial.print(sensorValue.un.magneticField.x);
      //   Serial.print("  Y: ");
      //   Serial.print(sensorValue.un.magneticField.y);
      //   Serial.print("  Z: ");
      //   Serial.print(sensorValue.un.magneticField.z);
      //   Serial.println(" uT");
      //   break;
    }
  }

  Serial.printf("P %f %f %f\n", position_x, position_y, position_z);
  // Serial.printf("GYRO: x %f, y %f, z %f\n", absolute_gyro_x, absolute_gyro_y, absolute_gyro_z);

  // If the BNO085 resets, its reports must be enabled again.
  if (bno08x.wasReset()) {
    Serial.println("BNO085 reset; re-enabling reports");
    enableReports();
  }
}


