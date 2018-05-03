#ifndef LIBSNIPS_NLU_H_
#define LIBSNIPS_NLU_H_

#ifdef __cplusplus
extern "C" {
#endif

typedef enum SNIPS_PRECISION {
    SNIPS_PRECISION_APPROXIMATE = 0,
    SNIPS_PRECISION_EXACT = 1,
} SNIPS_PRECISION;

typedef enum SNIPS_GRAIN {
    SNIPS_GRAIN_YEAR = 0,
    SNIPS_GRAIN_QUARTER = 1,
    SNIPS_GRAIN_MONTH = 2,
    SNIPS_GRAIN_WEEK = 3,
    SNIPS_GRAIN_DAY = 4,
    SNIPS_GRAIN_HOUR = 5,
    SNIPS_GRAIN_MINUTE = 6,
    SNIPS_GRAIN_SECOND = 7,
} SNIPS_GRAIN;

typedef enum SNIPS_SLOT_VALUE_TYPE {
    SNIPS_SLOT_VALUE_TYPE_CUSTOM = 1,
    SNIPS_SLOT_VALUE_TYPE_NUMBER = 2,
    SNIPS_SLOT_VALUE_TYPE_ORDINAL = 3,
    SNIPS_SLOT_VALUE_TYPE_INSTANTTIME = 4,
    SNIPS_SLOT_VALUE_TYPE_TIMEINTERVAL = 5,
    SNIPS_SLOT_VALUE_TYPE_AMOUNTOFMONEY = 6,
    SNIPS_SLOT_VALUE_TYPE_TEMPERATURE = 7,
    SNIPS_SLOT_VALUE_TYPE_DURATION = 8,
    SNIPS_SLOT_VALUE_TYPE_PERCENTAGE = 9,
} SNIPS_SLOT_VALUE_TYPE;

typedef double CNumberValue;

typedef double CPercentageValue;

typedef long COrdinalValue;

typedef struct CInstantTimeValue {
    char *const value;
    SNIPS_GRAIN grain;
    SNIPS_PRECISION precision;
} CInstantTimeValue;

typedef struct CTimeIntervalValue {
    char *const from;
    char *const to;
} CTimeIntervalValue;

typedef struct CAmountOfMoneyValue {
    float value;
    SNIPS_PRECISION precision;
    char *const unit;
} CAmountOfMoneyValue;

typedef struct CTemperatureValue {
    float value;
    char *const unit;
} CTemperatureValue;

typedef struct CDurationValue {
    long years;
    long quarters;
    long months;
    long weeks;
    long days;
    long hours;
    long minutes;
    long seconds;
    SNIPS_PRECISION precision;
} CDurationValue;

typedef struct CSlotValue {
    SNIPS_SLOT_VALUE_TYPE value_type;
    /**
      * Points to either a char *const, a CNumberValue, a COrdinalValue,
      * a CInstantTimeValue, a CTimeIntervalValue, a CAmountOfMoneyValue,
      * a CTemperatureValue or a CDurationValue depending on value_type
      */
    void *const value;
} CSlotValue;

typedef struct CSlot {
    char *const raw_value;
    CSlotValue value;
    int range_start;
    int range_end;
    char *const entity;
    char *const slot_name;
} CSlot;

typedef struct CIntentClassifierResult {
    char *const intent_name;
    float probability;
} CIntentClassifierResult;

typedef struct CSlotList {
    CSlot *const slots;
    int size;
} CSlotList;

typedef struct CIntentParserResult{
    char *const input;
    CIntentClassifierResult *const intent;
    CSlotList *const slots;
} CIntentParserResult;

typedef struct CSnipsNluEngine CSnipsNluEngine;

typedef enum SNIPS_RESULT {
	SNIPS_RESULT_OK = 0,
	SNIPS_RESULT_KO = 1,
} SNIPS_RESULT;

SNIPS_RESULT snips_nlu_engine_create_from_file(char const* file_path, CSnipsNluEngine** client);

SNIPS_RESULT snips_nlu_engine_create_from_dir(char const* root_dir, CSnipsNluEngine** client);

SNIPS_RESULT snips_nlu_engine_create_from_zip(unsigned char const* zip, unsigned int zip_size, CSnipsNluEngine** client);

SNIPS_RESULT snips_nlu_engine_run_parse(CSnipsNluEngine const* client, char const* input, CIntentParserResult** result);

SNIPS_RESULT snips_nlu_engine_run_parse_into_json(CSnipsNluEngine const* client, char const* input, char** result_json);

SNIPS_RESULT snips_nlu_engine_destroy_string(char* string);

SNIPS_RESULT snips_nlu_engine_destroy_client(CSnipsNluEngine* client);

SNIPS_RESULT snips_nlu_engine_destroy_result(CIntentParserResult* result);

SNIPS_RESULT snips_nlu_engine_get_last_error(char **error);

SNIPS_RESULT snips_nlu_engine_get_model_version(char **version);

#ifdef __cplusplus
}
#endif

#endif // !LIBSNIPS_NLU_H_
