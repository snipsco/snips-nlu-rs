package ai.snips.nlu

import ai.snips.nlu.ontology.IntentParserResult
import ai.snips.nlu.ontology.ffi.CIntentParserResult
import ai.snips.nlu.ontology.ffi.readString
import ai.snips.nlu.ontology.ffi.toPointer
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.ptr.PointerByReference
import java.io.Closeable
import java.io.File
import ai.snips.nlu.NluEngine.SnipsNluClientLibrary.Companion.INSTANCE as LIB

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

    val client: Pointer = clientBuilder()

    override fun close() {
        LIB.snips_nlu_engine_destroy_client(client)
    }

    fun parse(input: String): IntentParserResult =
            CIntentParserResult(PointerByReference().apply {
                parseError(LIB.snips_nlu_engine_run_parse(client, input.toPointer(), this))
            }.value).let {
                it.toIntentParserResult().apply {
                    // we don't want jna to try and sync this struct after the call as we're destroying it
                    // /!\ removing that will make the app crash semi randomly...
                    it.autoRead = false
                    LIB.snips_nlu_engine_destroy_result(it)
                }
            }

    fun parseIntoJson(input: String): String =
            PointerByReference().apply {
                parseError(LIB.snips_nlu_engine_run_parse_into_json(client, input.toPointer(), this))
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
        fun snips_nlu_engine_run_parse(client: Pointer, input: Pointer, result: PointerByReference): Int
        fun snips_nlu_engine_run_parse_into_json(client: Pointer, input: Pointer, result: PointerByReference): Int
        fun snips_nlu_engine_get_last_error(error: PointerByReference): Int
        fun snips_nlu_engine_destroy_client(client: Pointer): Int
        fun snips_nlu_engine_destroy_result(result: CIntentParserResult): Int
        fun snips_nlu_engine_destroy_string(string: Pointer): Int
    }
}
