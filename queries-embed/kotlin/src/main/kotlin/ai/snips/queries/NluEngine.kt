package ai.snips.queries

import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure
import com.sun.jna.ptr.PointerByReference
import java.io.Closeable
import java.io.File
import kotlin.system.measureTimeMillis


object Main {
    @JvmStatic
    fun main(args: Array<String>) {
        println("hello world")
        NluEngine(File("/home/fredszaq/Work/tmp/assistant")).apply {
            println("created")
            /* println("parse time 1 : " + measureTimeMillis {
                 println(parse("Make me a latte"))
             })*/
            println("parse time 2 : " + measureTimeMillis {
                println(parse("Can I have a medium sized decaf cappuccino with skimmed milk."))
                //println(parse("what's the weather like in paris ? "))
            })
        }.close()
        println("bye world")
    }
}

data class Range(val start: Int, val end: Int)

data class Slot(val value: String, val range: Range?, val entity: String, val slotName: String)
data class IntentClassifierResult(val intentName: String, val probability: Float)
data class IntentParserResult(val input: String, val intent: IntentClassifierResult?, val slots: List<Slot>)

class NluEngine(assistantDir: File) : Closeable {

    val client: Pointer = PointerByReference().apply {
        parseError(SnipsQueriesClientLibrary.INSTANCE.nlu_engine_create_from_dir(assistantDir.absolutePath, this))
    }.value

    override fun close() {
        SnipsQueriesClientLibrary.INSTANCE.nlu_engine_destroy_client(client)
    }

    fun parse(input: String): IntentParserResult =
            CIntentParserResult(PointerByReference().apply {
                parseError(SnipsQueriesClientLibrary.INSTANCE.nlu_engine_run_parse(client, input, this))
            }.value).let {
                it.toIntentParserResult().apply {
                    SnipsQueriesClientLibrary.INSTANCE.nlu_engine_destroy_result(it)
                }
            }

    private fun parseError(returnCode: Int) {
        if (returnCode != 1) {
            PointerByReference().apply {
                SnipsQueriesClientLibrary.INSTANCE.nlu_engine_get_last_error(this)
                throw RuntimeException(value.getString(0, "utf-8").apply {
                    SnipsQueriesClientLibrary.INSTANCE.nlu_engine_destroy_string(value)
                })
            }
        }
    }

    private interface SnipsQueriesClientLibrary : Library {
        companion object {
            val INSTANCE: SnipsQueriesClientLibrary = Native.loadLibrary("queries_embed", SnipsQueriesClientLibrary::class.java)
        }

        fun nlu_engine_create_from_dir(root_dir: String, pointer: PointerByReference): Int
        fun nlu_engine_run_parse(client: Pointer, input: String, result: PointerByReference): Int
        fun nlu_engine_get_last_error(error: PointerByReference): Int
        fun nlu_engine_destroy_client(client: Pointer): Int
        fun nlu_engine_destroy_result(result: CIntentParserResult): Int
        fun nlu_engine_destroy_string(string: Pointer): Int
    }


    class CIntentParserResult(p: Pointer) : Structure(p), Structure.ByReference {
        init {
            read()
        }

        @JvmField var input: String? = null
        @JvmField var intent: CIntentClassifierResult? = null
        @JvmField var slots: CSlots? = null

        override fun getFieldOrder() = listOf("input",
                                              "intent",
                                              "slots")

        fun toIntentParserResult() = IntentParserResult(input = input!!,
                                                        intent = intent?.toIntentClassifierResult(),
                                                        slots = slots?.toSlotList() ?: listOf())

    }

    class CIntentClassifierResult : Structure(), Structure.ByReference {
        @JvmField var intent_name: String? = null
        @JvmField var probability: Float? = null

        override fun getFieldOrder() = listOf("intent_name", "probability")

        fun toIntentClassifierResult() = IntentClassifierResult(intentName = intent_name!!, probability = probability!!)
    }

    class CSlots : Structure(), Structure.ByReference {

        @JvmField var slots: Pointer? = null
        @JvmField var size: Int = -1

        override fun getFieldOrder() = listOf("slots", "size")

        fun toSlotList(): List<Slot> =
                if (size > 0)
                    CSlot(slots!!).toArray(size).map { (it as CSlot).toSlot() }
                else listOf<Slot>()

    }

    class CSlot(p: Pointer) : Structure(p), Structure.ByReference {
        init {
            read()
        }

        @JvmField var value: String? = null
        @JvmField var range_start: Int? = null
        @JvmField var range_end: Int? = null
        @JvmField var entity: String? = null
        @JvmField var slot_name: String? = null

        override fun getFieldOrder() = listOf("value",
                                              "range_start",
                                              "range_end",
                                              "entity",
                                              "slot_name")

        fun toSlot() = Slot(value = value!!,
                            range = if (range_start != -1) Range(range_start!!, range_end!!) else null,
                            entity = entity!!,
                            slotName = slot_name!!)
    }
}
