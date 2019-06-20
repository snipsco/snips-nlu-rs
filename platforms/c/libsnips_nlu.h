#define SNIPS_NLU_VERSION "0.64.4"

#ifndef LIBSNIPS_NLU_H_
#define LIBSNIPS_NLU_H_

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Enum representing the grain of a resolved date related value
 */
typedef enum {
  /**
   * The resolved value has a granularity of a year
   */
  SNIPS_GRAIN_YEAR = 0,
  /**
   * The resolved value has a granularity of a quarter
   */
  SNIPS_GRAIN_QUARTER = 1,
  /**
   * The resolved value has a granularity of a mount
   */
  SNIPS_GRAIN_MONTH = 2,
  /**
   * The resolved value has a granularity of a week
   */
  SNIPS_GRAIN_WEEK = 3,
  /**
   * The resolved value has a granularity of a day
   */
  SNIPS_GRAIN_DAY = 4,
  /**
   * The resolved value has a granularity of an hour
   */
  SNIPS_GRAIN_HOUR = 5,
  /**
   * The resolved value has a granularity of a minute
   */
  SNIPS_GRAIN_MINUTE = 6,
  /**
   * The resolved value has a granularity of a second
   */
  SNIPS_GRAIN_SECOND = 7,
} SNIPS_GRAIN;

/**
 * Enum describing the precision of a resolved value
 */
typedef enum {
  /**
   * The resolved value is approximate
   */
  SNIPS_PRECISION_APPROXIMATE = 0,
  /**
   * The resolved value is exact
   */
  SNIPS_PRECISION_EXACT = 1,
} SNIPS_PRECISION;

/**
 * Used as a return type of functions that can encounter errors
 */
typedef enum {
  /**
   * The function returned successfully
   */
  SNIPS_RESULT_OK = 0,
  /**
   * The function encountered an error, you can retrieve it using the dedicated function
   */
  SNIPS_RESULT_KO = 1,
} SNIPS_RESULT;

/**
 * Enum type describing how to cast the value of a CSlotValue
 */
typedef enum {
  /**
   * Custom type represented by a char *
   */
  SNIPS_SLOT_VALUE_TYPE_CUSTOM = 1,
  /**
   * Number type represented by a CNumberValue
   */
  SNIPS_SLOT_VALUE_TYPE_NUMBER = 2,
  /**
   * Ordinal type represented by a COrdinalValue
   */
  SNIPS_SLOT_VALUE_TYPE_ORDINAL = 3,
  /**
   * Instant type represented by a CInstantTimeValue
   */
  SNIPS_SLOT_VALUE_TYPE_INSTANTTIME = 4,
  /**
   * Interval type represented by a CTimeIntervalValue
   */
  SNIPS_SLOT_VALUE_TYPE_TIMEINTERVAL = 5,
  /**
   * Amount of money type represented by a CAmountOfMoneyValue
   */
  SNIPS_SLOT_VALUE_TYPE_AMOUNTOFMONEY = 6,
  /**
   * Temperature type represented by a CTemperatureValue
   */
  SNIPS_SLOT_VALUE_TYPE_TEMPERATURE = 7,
  /**
   * Duration type represented by a CDurationValue
   */
  SNIPS_SLOT_VALUE_TYPE_DURATION = 8,
  /**
   * Percentage type represented by a CPercentageValue
   */
  SNIPS_SLOT_VALUE_TYPE_PERCENTAGE = 9,
  /**
   * Music Album type represented by a char *
   */
  SNIPS_SLOT_VALUE_TYPE_MUSICALBUM = 10,
  /**
   * Music Artist type represented by a char *
   */
  SNIPS_SLOT_VALUE_TYPE_MUSICARTIST = 11,
  /**
   * Music Track type represented by a char *
   */
  SNIPS_SLOT_VALUE_TYPE_MUSICTRACK = 12,
} SNIPS_SLOT_VALUE_TYPE;

typedef struct CSnipsNluEngine CSnipsNluEngine;

/**
 * Results of the intent classifier
 */
typedef struct {
  /**
   * Name of the intent detected
   */
  const char *intent_name;
  /**
   * Between 0 and 1
   */
  float confidence_score;
} CIntentClassifierResult;

/**
 * Wrapper around a list of IntentClassifierResult
 */
typedef struct {
  /**
   * Pointer to the first result of the list
   */
  const CIntentClassifierResult *intent_classifier_results;
  /**
   * Number of results in the list
   */
  int32_t size;
} CIntentClassifierResultList;

/**
 * A slot value
 */
typedef struct {
  /**
   * Points to either a *const char, a CNumberValue, a COrdinalValue,
   * a CInstantTimeValue, a CTimeIntervalValue, a CAmountOfMoneyValue,
   * a CTemperatureValue or a CDurationValue depending on value_type
   */
  const void *value;
  /**
   * The type of the value
   */
  SNIPS_SLOT_VALUE_TYPE value_type;
} CSlotValue;

/**
 * Struct describing a Slot
 */
typedef struct {
  /**
   * The resolved value of the slot
   */
  CSlotValue value;
  /**
   * The raw value as it appears in the input text
   */
  const char *raw_value;
  /**
   * Name of the entity type of the slot
   */
  const char *entity;
  /**
   * Name of the slot
   */
  const char *slot_name;
  /**
   * Start index of raw value in input text
   */
  int32_t range_start;
  /**
   * End index of raw value in input text
   */
  int32_t range_end;
  /**
   * Confidence score of the slot
   */
  float confidence_score;
} CSlot;

/**
 * Wrapper around a slot list
 */
typedef struct {
  /**
   * Pointer to the first slot of the list
   */
  const CSlot *slots;
  /**
   * Number of slots in the list
   */
  int32_t size;
} CSlotList;

/**
 * Result of intent parsing
 */
typedef struct {
  /**
   * The text that was parsed
   */
  const char *input;
  /**
   * The result of intent classification
   */
  const CIntentClassifierResult *intent;
  /**
   * The slots extracted
   */
  const CSlotList *slots;
} CIntentParserResult;

/**
 * An array of strings
 */
typedef struct {
  /**
   * Pointer to the first element of the array
   */
  const char *const *data;
  /**
   * Number of elements in the array
   */
  int size;
} CStringArray;

/**
 * Representation of a number value
 */
typedef double CNumberValue;

/**
 * Representation of an ordinal value
 */
typedef int64_t COrdinalValue;

/**
 * Representation of a percentage value
 */
typedef double CPercentageValue;

/**
 * Representation of an instant value
 */
typedef struct {
  /**
   * String representation of the instant
   */
  const char *value;
  /**
   * The grain of the resolved instant
   */
  SNIPS_GRAIN grain;
  /**
   * The precision of the resolved instant
   */
  SNIPS_PRECISION precision;
} CInstantTimeValue;

/**
 * Representation of an interval value
 */
typedef struct {
  /**
   * String representation of the beginning of the interval
   */
  const char *from;
  /**
   * String representation of the end of the interval
   */
  const char *to;
} CTimeIntervalValue;

/**
 * Representation of an amount of money value
 */
typedef struct {
  /**
   * The currency
   */
  const char *unit;
  /**
   * The amount of money
   */
  float value;
  /**
   * The precision of the resolved value
   */
  SNIPS_PRECISION precision;
} CAmountOfMoneyValue;

/**
 * Representation of a temperature value
 */
typedef struct {
  /**
   * The unit used
   */
  const char *unit;
  /**
   * The temperature resolved
   */
  float value;
} CTemperatureValue;

/**
 * Representation of a duration value
 */
typedef struct {
  /**
   * Number of years in the duration
   */
  int64_t years;
  /**
   * Number of quarters in the duration
   */
  int64_t quarters;
  /**
   * Number of months in the duration
   */
  int64_t months;
  /**
   * Number of weeks in the duration
   */
  int64_t weeks;
  /**
   * Number of days in the duration
   */
  int64_t days;
  /**
   * Number of hours in the duration
   */
  int64_t hours;
  /**
   * Number of minutes in the duration
   */
  int64_t minutes;
  /**
   * Number of seconds in the duration
   */
  int64_t seconds;
  /**
   * Precision of the resolved value
   */
  SNIPS_PRECISION precision;
} CDurationValue;

SNIPS_RESULT snips_nlu_engine_create_from_dir(const char *root_dir, const CSnipsNluEngine **client);

SNIPS_RESULT snips_nlu_engine_create_from_zip(const unsigned char *zip,
                                              unsigned int zip_size,
                                              const CSnipsNluEngine **client);

SNIPS_RESULT snips_nlu_engine_destroy_client(CSnipsNluEngine *client);

SNIPS_RESULT snips_nlu_engine_destroy_intent_classifier_results(CIntentClassifierResultList *result);

SNIPS_RESULT snips_nlu_engine_destroy_result(CIntentParserResult *result);

SNIPS_RESULT snips_nlu_engine_destroy_slots(CSlotList *result);

SNIPS_RESULT snips_nlu_engine_destroy_string(char *string);

/**
 * Used to retrieve the last error that happened in this thread. A function encountered an
 * error if its return type is of type SNIPS_RESULT and it returned SNIPS_RESULT_KO
 */
SNIPS_RESULT snips_nlu_engine_get_last_error(const char **error);

SNIPS_RESULT snips_nlu_engine_get_model_version(const char **version);

SNIPS_RESULT snips_nlu_engine_run_get_intents(const CSnipsNluEngine *client,
                                              const char *input,
                                              const CIntentClassifierResultList **result);

SNIPS_RESULT snips_nlu_engine_run_get_intents_into_json(const CSnipsNluEngine *client,
                                                        const char *input,
                                                        const char **result_json);

SNIPS_RESULT snips_nlu_engine_run_get_slots(const CSnipsNluEngine *client,
                                            const char *input,
                                            const char *intent,
                                            const CSlotList **result);

SNIPS_RESULT snips_nlu_engine_run_get_slots_into_json(const CSnipsNluEngine *client,
                                                      const char *input,
                                                      const char *intent,
                                                      const char **result_json);

SNIPS_RESULT snips_nlu_engine_run_parse(const CSnipsNluEngine *client,
                                        const char *input,
                                        const CStringArray *intents_whitelist,
                                        const CStringArray *intents_blacklist,
                                        const CIntentParserResult **result);

SNIPS_RESULT snips_nlu_engine_run_parse_into_json(const CSnipsNluEngine *client,
                                                  const char *input,
                                                  const CStringArray *intents_whitelist,
                                                  const CStringArray *intents_blacklist,
                                                  const char **result_json);

#endif /* LIBSNIPS_NLU_H_ */
