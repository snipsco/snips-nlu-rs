package ai.snips.nlu

import ai.snips.nlu.ontology.IntentParserResult
import ai.snips.nlu.ontology.IntentClassifierResult
import ai.snips.nlu.ontology.Slot
import ai.snips.nlu.ontology.ffi.CIntentParserResult
import ai.snips.nlu.ontology.ffi.CIntentClassifierResultList
import ai.snips.nlu.ontology.ffi.CSlots
import ai.snips.nlu.ontology.ffi.readString
import ai.snips.nlu.ontology.ffi.toPointer
import com.sun.jna.Library
import com.sun.jna.Memory
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.ptr.PointerByReference
import com.sun.jna.Structure
import com.sun.jna.toJnaPointer
import java.io.Closeable
import java.io.File
import ai.snips.nlu.NluEngine.SnipsNluClientLibrary.Companion.INSTANCE as LIB

class CStringArray(p: Pointer?) : Structure(p), Structure.ByReference {
    companion object {
        @JvmStatic
        fun fromStringList(list: List<String>) = CStringArray(null).apply {
            size = list.size
            data = if (size > 0)
                Memory(Pointer.SIZE * list.size.toLong()).apply {
                    list.forEachIndexed { i, s ->
                        this.setPointer(i.toLong() * Pointer.SIZE, s.toPointer())
                    }
                }
            else null
        }
    }

    @JvmField
    var data: Pointer? = null
    @JvmField
    var size: Int = -1

    // be careful this block must be below the field definition if you don't want the native values read by JNA
    // overridden by the default ones
    init {
        read()
    }

    override fun getFieldOrder() = listOf("data", "size")

    fun toStringList() = if (size > 0) {
        data!!.getPointerArray(0, size).map { it.readString() }
    } else listOf<String>()
}

class NluEngine private constructor(clientBuilder: () -> Pointer) : Closeable {

    companion object {
        private fun parseError(returnCode: Int) {
            if (returnCode != 0) {
                PointerByReference().apply {
                    LIB.snips_nlu_engine_get_last_error(this)
                    throw RuntimeException(value.getString(0).apply {
                        LIB.snips_nlu_engine_destroy_string(value)
                    })
                }
            }
        }

        @JvmStatic
        fun modelVersion(): String = PointerByReference().run {
            parseError(LIB.snips_nlu_engine_get_model_version(this))
            value.getString(0).apply { LIB.snips_nlu_engine_destroy_string(value) }
        }
    }

    constructor(assistant: File) :
            this({
                     PointerByReference().apply {
                         parseError(LIB.snips_nlu_engine_create_from_dir(assistant.absolutePath.toPointer(), this))
                     }.value
                 })

    constructor(data: ByteArray) :
            this({
                PointerByReference().apply {
                    parseError(LIB.snips_nlu_engine_create_from_zip(data, data.size, this))
                }.value
            })

    val client: Pointer = clientBuilder()

    override fun close() {
        LIB.snips_nlu_engine_destroy_client(client)
    }

    fun parse(input: String,
              intentsWhitelist: List<String>? = null,
              intentsBlacklist: List<String>? = null): IntentParserResult =
            CIntentParserResult(PointerByReference().apply {
                parseError(LIB.snips_nlu_engine_run_parse(
                        client,
                        input.toPointer(),
                        intentsWhitelist?.let { CStringArray.fromStringList(it) },
                        intentsBlacklist?.let { CStringArray.fromStringList(it) },
                        this
                ))
            }.value).let {
                it.toIntentParserResult().apply {
                    // we don't want jna to try and sync this struct after the call as we're destroying it
                    // /!\ removing that will make the app crash semi randomly...
                    it.autoRead = false
                    LIB.snips_nlu_engine_destroy_result(it)
                }
            }

    fun parseIntoJson(input: String,
                      intentsWhitelist: List<String>? = null,
                      intentsBlacklist: List<String>? = null): String =
            PointerByReference().apply {
                parseError(LIB.snips_nlu_engine_run_parse_into_json(
                        client,
                        input.toPointer(),
                        intentsWhitelist?.let { CStringArray.fromStringList(it) },
                        intentsBlacklist?.let { CStringArray.fromStringList(it) },
                        this
                ))
            }.value.let {
                it.readString().apply {
                    LIB.snips_nlu_engine_destroy_string(it)
                }
            }

    fun getSlots(input: String, intent: String): List<Slot> =
            CSlots(PointerByReference().apply {
                parseError(LIB.snips_nlu_engine_run_get_slots(
                        client,
                        input.toPointer(),
                        intent.toPointer(),
                        this
                ))
            }.value).let {
                it.toSlotList().apply {
                    // we don't want jna to try and sync this struct after the call as we're destroying it
                    // /!\ removing that will make the app crash semi randomly...
                    it.autoRead = false
                    LIB.snips_nlu_engine_destroy_slots(it)
                }
            }

    fun getSlotsIntoJson(input: String, intent: String): String =
            PointerByReference().apply {
                parseError(LIB.snips_nlu_engine_run_get_slots_into_json(
                        client,
                        input.toPointer(),
                        intent.toPointer(),
                        this
                ))
            }.value.let {
                it.readString().apply {
                    LIB.snips_nlu_engine_destroy_string(it)
                }
            }

    fun getIntents(input: String): List<IntentClassifierResult> =
            CIntentClassifierResultList(PointerByReference().apply {
                parseError(LIB.snips_nlu_engine_run_get_intents(client, input.toPointer(), this))
            }.value).let {
                it.toIntentClassifierResultList().apply {
                    // we don't want jna to try and sync this struct after the call as we're destroying it
                    // /!\ removing that will make the app crash semi randomly...
                    it.autoRead = false
                    LIB.snips_nlu_engine_destroy_intent_classifier_results(it)
                }
            }

    fun getIntentsIntoJson(input: String): String =
            PointerByReference().apply {
                parseError(LIB.snips_nlu_engine_run_get_intents_into_json(
                        client,
                        input.toPointer(),
                        this
                ))
            }.value.let {
                it.readString().apply {
                    LIB.snips_nlu_engine_destroy_string(it)
                }
            }

    internal interface SnipsNluClientLibrary : Library {
        companion object {
            val INSTANCE: SnipsNluClientLibrary = Native.loadLibrary("snips_nlu_ffi", SnipsNluClientLibrary::class.java)
        }

        fun snips_nlu_engine_get_model_version(version: PointerByReference): Int
        fun snips_nlu_engine_create_from_dir(root_dir: Pointer, pointer: PointerByReference): Int
        fun snips_nlu_engine_create_from_zip(data: ByteArray, data_size: Int, pointer: PointerByReference): Int
        fun snips_nlu_engine_run_parse(
                client: Pointer, input: Pointer,
                intents_whitelist: CStringArray?,
                intents_blacklist: CStringArray?,
                result: PointerByReference): Int
        fun snips_nlu_engine_run_parse_into_json(
                client: Pointer,
                input: Pointer,
                intents_whitelist: CStringArray?,
                intents_blacklist: CStringArray?,
                result: PointerByReference): Int
        fun snips_nlu_engine_run_get_slots(
                client: Pointer,
                input: Pointer,
                intent: Pointer,
                result: PointerByReference): Int
        fun snips_nlu_engine_run_get_slots_into_json(
                client: Pointer,
                input: Pointer,
                intent: Pointer,
                result: PointerByReference): Int
        fun snips_nlu_engine_run_get_intents(
                client: Pointer,
                input: Pointer,
                result: PointerByReference): Int
        fun snips_nlu_engine_run_get_intents_into_json(
                client: Pointer,
                input: Pointer,
                result: PointerByReference): Int
        fun snips_nlu_engine_get_last_error(error: PointerByReference): Int
        fun snips_nlu_engine_destroy_client(client: Pointer): Int
        fun snips_nlu_engine_destroy_result(result: CIntentParserResult): Int
        fun snips_nlu_engine_destroy_slots(result: CSlots): Int
        fun snips_nlu_engine_destroy_intent_classifier_results(result: CIntentClassifierResultList): Int
        fun snips_nlu_engine_destroy_string(string: Pointer): Int
    }
}
