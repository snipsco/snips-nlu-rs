package ai.snips.queries

import ai.snips.queries.ontology.IntentParserResult
import ai.snips.queries.ontology.ffi.CIntentParserResult
import ai.snips.queries.ontology.ffi.readString
import ai.snips.queries.ontology.ffi.toPointer
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.ptr.PointerByReference
import java.io.Closeable
import java.io.File
import ai.snips.queries.NluEngine.SnipsQueriesClientLibrary.Companion.INSTANCE as LIB

class NluEngine private constructor(clientBuilder: () -> Pointer) : Closeable {

    companion object {
        private fun parseError(returnCode: Int) {
            if (returnCode != 1) {
                PointerByReference().apply {
                    LIB.nlu_engine_get_last_error(this)
                    throw RuntimeException(value.getString(0).apply {
                        LIB.nlu_engine_destroy_string(value)
                    })
                }
            }
        }

        @JvmStatic
        fun modelVersion(): String = PointerByReference().run {
            parseError(LIB.nlu_engine_get_model_version(this))
            value.getString(0).apply { LIB.nlu_engine_destroy_string(value) }
        }
    }

    constructor(assistantDir: File) :
            this({
                     PointerByReference().apply {
                         parseError(LIB.nlu_engine_create_from_dir(assistantDir.absolutePath.toPointer(), this))
                     }.value
                 })

    constructor(data: ByteArray) :
            this({
                     PointerByReference().apply {
                         parseError(LIB.nlu_engine_create_from_zip(data, data.size, this))
                     }.value
                 })


    val client: Pointer = clientBuilder()

    override fun close() {
        LIB.nlu_engine_destroy_client(client)
    }

    fun parse(input: String): IntentParserResult =
            CIntentParserResult(PointerByReference().apply {
                parseError(LIB.nlu_engine_run_parse(client, input.toPointer(), this))
            }.value).let {
                it.toIntentParserResult().apply {
                    // we don't want jna to try and sync this struct after the call as we're destroying it
                    // /!\ removing that will make the app crash semi randomly...
                    it.autoRead = false
                    LIB.nlu_engine_destroy_result(it)
                }
            }

    fun parseIntoJson(input: String): String =
            PointerByReference().apply {
                parseError(LIB.nlu_engine_run_parse_into_json(client, input.toPointer(), this))
            }.value.let {
                it.readString().apply {
                    LIB.nlu_engine_destroy_string(it)
                }
            }

    internal interface SnipsQueriesClientLibrary : Library {
        companion object {
            val INSTANCE: SnipsQueriesClientLibrary = Native.loadLibrary("snips_queries", SnipsQueriesClientLibrary::class.java)
        }

        fun nlu_engine_get_model_version(version: PointerByReference): Int
        fun nlu_engine_create_from_dir(root_dir: Pointer, pointer: PointerByReference): Int
        fun nlu_engine_create_from_zip(data: ByteArray, data_size: Int, pointer: PointerByReference): Int
        fun nlu_engine_run_parse(client: Pointer, input: Pointer, result: PointerByReference): Int
        fun nlu_engine_run_parse_into_json(client: Pointer, input: Pointer, result: PointerByReference): Int
        fun nlu_engine_get_last_error(error: PointerByReference): Int
        fun nlu_engine_destroy_client(client: Pointer): Int
        fun nlu_engine_destroy_result(result: CIntentParserResult): Int
        fun nlu_engine_destroy_string(string: Pointer): Int
    }
}
