#ifndef LIBQUERIES_EMBED_H_
#define LIBQUERIES_EMBED_H_

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>

typedef struct CSlot {
    char *const value;
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
    CSlot* slots;
    int size;
} CSlotList;

typedef struct CIntentParserResult{
    char *const input;
    CIntentClassifierResult *intent;
    CSlotList* slots;
} CIntentParserResult;

typedef struct Opaque Opaque;

typedef enum QUERIESRESULT {
	KO = 0,
	OK = 1,
} QUERIESRESULT;

QUERIESRESULT nlu_engine_create_from_dir(char const* root_dir, Opaque** client);

QUERIESRESULT nlu_engine_create_from_binary(unsigned char const* bytes, unsigned int binary_size, Opaque** client);

QUERIESRESULT nlu_engine_run_parse(Opaque* client, char const* input, CIntentParserResult** result);

QUERIESRESULT nlu_engine_run_parse_into_json(Opaque* client, char const* input, char** result_json);

QUERIESRESULT nlu_engine_destroy_string(char* string);

QUERIESRESULT nlu_engine_destroy_client(Opaque* client);

QUERIESRESULT nlu_engine_destroy_result(CIntentParserResult* result);

QUERIESRESULT nlu_engine_get_last_error(char **error);

#ifdef __cplusplus
}
#endif

#endif // !LIBQUERIES_EMBED_H_
