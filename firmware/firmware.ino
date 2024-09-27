#include <SPI.h>
#include "ADS131M04.h"
#include "fircoef.h"

class FIRFilter
{
public:
  FIRFilter() = default;

  float filter(float sample)
  {
    _window[_index] = sample;
    _index = (_index + 1) % kNumCoefficients;

    float result = 0;
    for (size_t i = 0; i < kNumCoefficients; i++)
    {
      result += kFIRCoefficients[i] * _window[(_index + i) % kNumCoefficients];
    }

    return result;
  }
private:
    size_t _index = 0;

    static constexpr size_t kNumCoefficients = sizeof(kFIRCoefficients) / sizeof(kFIRCoefficients[0]);
    float _window[kNumCoefficients] = {0};
};

class RMSFilter
{
public:
  RMSFilter() = default;

  float filter(float sample)
  {
    _window[_index] = sample * sample;
    _index = (_index + 1) % kNumSamples;

    double sum = 0;
    for (size_t i = 0; i < kNumSamples; i++)
    {
      sum += _window[i];
    }

    return static_cast<float>(sqrt(sum / kNumSamples));
  }

private:
    size_t _index = 0;

    static constexpr size_t kNumSamples = 50;
    float _window[kNumSamples] = {0};
};

constexpr int kDLVRInterruptPin = 1;
constexpr int kCSAccelPin = 31;

constexpr int kADCChipSelectPin = 37;
constexpr int kADCDataReadyPin = 35;
constexpr int kADCSyncResetPin = 36;
constexpr int kADCSCKPin = 13;
constexpr int kADCMOSIPin = 11;
constexpr int kADCMISOPin = 12;

constexpr size_t kWindowSize = 1000;
int32_t _averages[3][1000];
size_t _averageCount = 0;

ADS131M04 _adc;

void configureOutputPin(int pin, int defaultLevel)
{
  pinMode(pin, OUTPUT);
  digitalWrite(pin, defaultLevel);
};

void setup()
{
  configureOutputPin(kCSAccelPin, 1);
  configureOutputPin(kADCChipSelectPin, 1);
  configureOutputPin(kADCSyncResetPin, 1);

  pinMode(kADCDataReadyPin, INPUT);

  // Allow the ADC to startup (it needs 0.5ms, so we give it 2)
  delay(2);

  _adc.begin(kADCChipSelectPin, kADCDataReadyPin);
  bool ok = _adc.setInputChannelSelection(0, INPUT_CHANNEL_MUX_AIN0P_AIN0N);
  ok &= _adc.setInputChannelSelection(1, INPUT_CHANNEL_MUX_AIN0P_AIN0N);
  ok &= _adc.setInputChannelSelection(2, INPUT_CHANNEL_MUX_AIN0P_AIN0N);
  ok &= _adc.setInputChannelSelection(3, INPUT_CHANNEL_MUX_AIN0P_AIN0N);
  ok &= _adc.setChannelPGA(0, CHANNEL_PGA_64);
  ok &= _adc.setChannelPGA(1, CHANNEL_PGA_64);
  ok &= _adc.setChannelPGA(2, CHANNEL_PGA_64);
  ok &= _adc.setChannelPGA(3, CHANNEL_PGA_64);

  ok &= _adc.setOsr(OSR_4096); // OSR_4096 == 1kHz with 8.192 MHz clock

  // TODO: handle error
}

void loop()
{
  delay(100);
  uint32_t prevTime = 0;

  FIRFilter firX;
  FIRFilter firY;
  FIRFilter firZ;

  const float VREF = 1.2;
  const float VOLTS_PER_COUNT = VREF / 0x7FFFFF;

  struct StatusReg {
    bool DRDY0 : 1;
    bool DRDY1 : 1;
    bool DRDY2 : 1;
    bool DRDY3 : 1;
    uint8_t reserved : 4;
    uint8_t wlength : 2;
    bool RESET : 1;
    bool CRC_TYPE : 1;
    bool CRC_ERR : 1;
    bool REG_MAP : 1;
    bool F_RESYNC : 1;
    bool lock : 1;
  };

  union {
    StatusReg reg;
    uint16_t value;
  } status;

  RMSFilter rmsFilterX;
  RMSFilter rmsFilterY;
  RMSFilter rmsFilterZ;

  while (1)
  {
    if (_adc.isDataReady())
    {
      auto res = _adc.readADC();

      auto ground = res.ch3;
      res.ch0 -= ground;
      res.ch1 -= ground;
      res.ch2 -= ground;

      auto x = firX.filter(res.ch0);
      auto y = firY.filter(res.ch1);
      auto z = firZ.filter(res.ch2);
      
      auto now = millis();
      constexpr uint32_t kIntervalMs = 10;

      if (now - prevTime >= kIntervalMs)
      {
        prevTime = now;
        status.value = res.status;

        Serial.print(micros());
        Serial.print(',');
        Serial.print(static_cast <int32_t>(x));
        Serial.print(',');
        Serial.print(static_cast <int32_t>(y));
        Serial.print(',');
        Serial.print(static_cast <int32_t>(z));
        Serial.print(',');
        Serial.print(res.ch0);
        Serial.print(',');
        Serial.print(res.ch1);
        Serial.print(',');
        Serial.print(res.ch2);
        Serial.println();
      }
    }
  }
}
