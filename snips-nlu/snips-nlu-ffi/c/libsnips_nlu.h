#ifndef LIBSNIPS_NLU_H_
#define LIBSNIPS_NLU_H_

#ifdef __cplusplus
extern "C" {
#endif

typedef enum CPrecision {
    APPROXIMATE = 0,
    EXACT = 1,
} CPrecision;

typedef enum CGrain {
    YEAR = 0,
    QUARTER = 1,
    MONTH = 2,
    WEEK = 3,
    DAY = 4,
    HOUR = 5,
    MINUTE = 6,
    SECOND = 7,
} CGrain;

typedef enum CSlotValueType {
    CUSTOM = 1,
    NUMBER = 2,
    ORDINAL = 3,
    INSTANTTIME = 4,
    TIMEINTERVAL = 5,
    AMOUNTOFMONEY = 6,
    TEMPERATURE = 7,
    DURATION = 8,
    PERCENTAGE = 9,
} CSlotValueType;

typedef double CNumberValue;

typedef double CPercentageValue;

typedef long COrdinalValue;

typedef struct CInstantTimeValue {
    char *const value;
    CGrain grain;
    CPrecision precision;
} CInstantTimeValue;

typedef struct CTimeIntervalValue {
    char *const from;
    char *const to;
} CTimeIntervalValue;

typedef struct CAmountOfMoneyValue {
    float value;
    CPrecision precision;
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
    CPrecision precision;
} CDurationValue;

typedef struct CSlotValue {
    CSlotValueType value_type;
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

typedef struct Opaque Opaque;

typedef enum NLURESULT {
	KO = 0,
	OK = 1,
} NLURESULT;

NLURESULT nlu_engine_create_from_dir(char const* root_dir, Opaque** client);

NLURESULT nlu_engine_create_from_zip(unsigned char const* zip, unsigned int zip_size, Opaque** client);

NLURESULT nlu_engine_run_parse(Opaque const* client, char const* input, CIntentParserResult** result);

NLURESULT nlu_engine_run_parse_into_json(Opaque const* client, char const* input, char** result_json);

NLURESULT nlu_engine_destroy_string(char* string);

NLURESULT nlu_engine_destroy_client(Opaque* client);

NLURESULT nlu_engine_destroy_result(CIntentParserResult* result);

NLURESULT nlu_engine_get_last_error(char **error);

NLURESULT nlu_engine_get_model_version(char **version);

#ifdef __cplusplus
}
#endif

#endif // !LIBSNIPS_NLU_H_
