package ai.snips.queries

import ai.snips.queries.SlotValue.AmountOfMoneyValue
import ai.snips.queries.SlotValue.CustomValue
import ai.snips.queries.SlotValue.DurationValue
import ai.snips.queries.SlotValue.InstantTimeValue
import ai.snips.queries.SlotValue.NumberValue
import ai.snips.queries.SlotValue.OrdinalValue
import ai.snips.queries.SlotValue.TemperatureValue
import ai.snips.queries.SlotValue.TimeIntervalValue
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure
import com.sun.jna.ptr.PointerByReference
import java.io.Closeable
import java.io.File
import kotlin.system.measureTimeMillis
import ai.snips.queries.NluEngine.SnipsQueriesClientLibrary.Companion.INSTANCE as LIB

object Main {
    @JvmStatic
    fun main(args: Array<String>) {
        println("hello world")
        println(NluEngine.modelVersion())
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

        NluEngine(File("/home/fredszaq/Work/tmp/assistantproj_SJvHP5PHQb/assistant.zip").readBytes()).apply {
            println(parse("Set the color of the lights to blue"))
            println(tag("Set the color of the lights to blue", "ActivateLightColor"))

        }
        println("bye world")
    }
}

data class Range(val start: Int, val end: Int)

data class Slot(val rawValue: String, val value: SlotValue, val range: Range?, val entity: String, val slotName: String)

enum class Precision {APPROXIMATE, EXACT }
enum class Grain { YEAR, QUARTER, MONTH, WEEK, DAY, HOUR, MINUTE, SECOND }

// TODO : add converters to JSR310 / ThreeTen types
// TODO : add type discriminant for Java API
sealed class SlotValue {
    data class CustomValue(val value: String) : SlotValue()
    data class NumberValue(val value: Double) : SlotValue()
    data class OrdinalValue(val value: Long) : SlotValue()
    data class InstantTimeValue(val value: String, val grain: Grain, val precision: Precision) : SlotValue()
    data class TimeIntervalValue(val from: String, val to: String) : SlotValue()
    data class AmountOfMoneyValue(val value: Float, val precision: Precision, val unit: String) : SlotValue()
    data class TemperatureValue(val value: Float, val unit: String) : SlotValue()
    data class DurationValue(val years: Long,
                             val quarters: Long,
                             val months: Long,
                             val weeks: Long,
                             val days: Long,
                             val hours: Long,
                             val minutes: Long,
                             val seconds: Long,
                             val precision: Precision) : SlotValue()
}


data class IntentClassifierResult(val intentName: String, val probability: Float)
data class IntentParserResult(val input: String, val intent: IntentClassifierResult?, val slots: List<Slot>)
data class TaggedEntity(val value: String, val range: Range?, val entity: String, val slotName: String)

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
                         parseError(LIB.nlu_engine_create_from_dir(assistantDir.absolutePath, this))
                     }.value
                 })

    constructor(data: ByteArray) :
            this({
                     PointerByReference().apply {
                         parseError(LIB.nlu_engine_create_from_binary(data, data.size, this))
                     }.value
                 })


    val client: Pointer = clientBuilder()

    override fun close() {
        LIB.nlu_engine_destroy_client(client)
    }

    fun parse(input: String): IntentParserResult =
            CIntentParserResult(PointerByReference().apply {
                parseError(LIB.nlu_engine_run_parse(client, input, this))
            }.value).let {
                it.toIntentParserResult().apply {
                    LIB.nlu_engine_destroy_result(it)
                }
            }

    fun tag(input: String, intent: String): List<TaggedEntity> =
            CTaggedEntities(PointerByReference().apply {
                parseError(LIB.nlu_engine_run_tag(client, input, intent, this))
            }.value).let {
                it.toTaggedEntityList().apply {
                    LIB.nlu_engine_destroy_tagged_entity_list(it)
                }
            }

    internal interface SnipsQueriesClientLibrary : Library {
        companion object {
            val INSTANCE: SnipsQueriesClientLibrary = Native.loadLibrary("snips_queries", SnipsQueriesClientLibrary::class.java)
        }

        fun nlu_engine_get_model_version(version: PointerByReference): Int
        fun nlu_engine_create_from_dir(root_dir: String, pointer: PointerByReference): Int
        fun nlu_engine_create_from_binary(data: ByteArray, data_size: Int, pointer: PointerByReference): Int
        fun nlu_engine_run_parse(client: Pointer, input: String, result: PointerByReference): Int
        fun nlu_engine_run_tag(client: Pointer, input: String, intent: String, result: PointerByReference): Int
        fun nlu_engine_get_last_error(error: PointerByReference): Int
        fun nlu_engine_destroy_client(client: Pointer): Int
        fun nlu_engine_destroy_result(result: CIntentParserResult): Int
        fun nlu_engine_destroy_tagged_entity_list(result: CTaggedEntities): Int
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

    object CGrain {
        const val YEAR = 0
        const val QUARTER = 1
        const val MONTH = 2
        const val WEEK = 3
        const val DAY = 4
        const val HOUR = 5
        const val MINUTE = 6
        const val SECOND = 7

        fun toGrain(input: Int) = when (input) {
            YEAR -> Grain.YEAR
            QUARTER -> Grain.QUARTER
            MONTH -> Grain.MONTH
            WEEK -> Grain.WEEK
            DAY -> Grain.DAY
            HOUR -> Grain.HOUR
            MINUTE -> Grain.MINUTE
            SECOND -> Grain.SECOND
            else -> throw IllegalArgumentException("unknown grain $input")
        }
    }

    object CPrecision {
        const val APPROXIMATE = 0
        const val EXACT = 1

        fun toPrecision(input: Int) = when (input) {
            APPROXIMATE -> Precision.APPROXIMATE
            EXACT -> Precision.EXACT
            else -> throw IllegalArgumentException("unknown precision $input")
        }
    }

    class CSlotValue : Structure(), Structure.ByValue {
        companion object {
            const val CUSTOM = 1
            const val NUMBER = 2
            const val ORDINAL = 3
            const val INSTANTTIME = 4
            const val TIMEINTERVAL = 5
            const val AMOUNTOFMONEY = 6
            const val TEMPERATURE = 7
            const val DURATION = 8


        }

        @JvmField var value_type: Int? = null
        @JvmField var value: Pointer? = null

        override fun getFieldOrder() = listOf("value_type", "value")

        fun toSlotValue(): SlotValue = when (value_type!!) {
            CUSTOM -> CustomValue(value!!.getString(0))
            NUMBER -> NumberValue(value!!.getDouble(0))
            ORDINAL -> OrdinalValue(value!!.getLong(0))
            INSTANTTIME -> CInstantTimeValue(value!!).toInstantTimeValue()
            TIMEINTERVAL -> CTimeIntervalValue(value!!).toTimeIntervalValue()
            AMOUNTOFMONEY -> CAmountOfMoneyValue(value!!).toAmountOfMoneyValue()
            TEMPERATURE -> CTemperatureValue(value!!).toTemperatureValue()
            DURATION -> CDurationValue(value!!).toDurationValue()
            else -> throw IllegalArgumentException("unknown value type $value_type")
        }

    }

    class CInstantTimeValue(p: Pointer) : Structure(p), Structure.ByReference {
        init {
            read()
        }

        @JvmField var value: String? = null
        @JvmField var grain: Int? = null
        @JvmField var precision: Int? = null
        override fun getFieldOrder() = listOf("value", "grain", "precision")
        fun toInstantTimeValue(): InstantTimeValue {
            return InstantTimeValue(value = value!!,
                                    grain = CGrain.toGrain(grain!!),
                                    precision = CPrecision.toPrecision(precision!!))

        }
    }

    class CTimeIntervalValue(p: Pointer) : Structure(p), Structure.ByReference {
        init {
            read()
        }

        @JvmField var from: String? = null
        @JvmField var to: String? = null
        override fun getFieldOrder() = listOf("from", "to")
        fun toTimeIntervalValue() = TimeIntervalValue(from = from!!, to = to!!)
    }

    class CAmountOfMoneyValue(p: Pointer) : Structure(p), Structure.ByReference {
        init {
            read()
        }

        @JvmField var value: Float? = null
        @JvmField var precision: Int? = null
        @JvmField var unit: String? = null
        override fun getFieldOrder() = listOf("value", "precision", "unit")
        fun toAmountOfMoneyValue() = AmountOfMoneyValue(value = value!!,
                                                        precision = CPrecision.toPrecision(precision!!),
                                                        unit = unit!!)
    }

    class CTemperatureValue(p: Pointer) : Structure(p), Structure.ByReference {
        init {
            read()
        }

        @JvmField var value: Float? = null
        @JvmField var unit: String? = null
        override fun getFieldOrder() = listOf("value", "unit")
        fun toTemperatureValue() = TemperatureValue(value = value!!,
                                                    unit = unit!!)

    }

    class CDurationValue(p: Pointer) : Structure(p), Structure.ByReference {
        init {
            read()
        }

        @JvmField var years: Long? = null
        @JvmField var quarters: Long? = null
        @JvmField var months: Long? = null
        @JvmField var weeks: Long? = null
        @JvmField var days: Long? = null
        @JvmField var hours: Long? = null
        @JvmField var minutes: Long? = null
        @JvmField var seconds: Long? = null
        @JvmField var precision: Int? = null


        override fun getFieldOrder() = listOf("years",
                                              "quarters",
                                              "months",
                                              "weeks",
                                              "days",
                                              "hours",
                                              "minutes",
                                              "seconds",
                                              "precision")

        fun toDurationValue() = DurationValue(years = years!!,
                                              quarters = quarters!!,
                                              months = months!!,
                                              weeks = weeks!!,
                                              days = days!!,
                                              hours = hours!!,
                                              minutes = minutes!!,
                                              seconds = seconds!!,
                                              precision = CPrecision.toPrecision(precision!!))
    }


    class CSlot(p: Pointer) : Structure(p), Structure.ByReference {
        init {
            read()
        }

        @JvmField var raw_value: String? = null
        @JvmField var value: CSlotValue? = null
        @JvmField var range_start: Int? = null
        @JvmField var range_end: Int? = null
        @JvmField var entity: String? = null
        @JvmField var slot_name: String? = null

        override fun getFieldOrder() = listOf("raw_value",
                                              "value",
                                              "range_start",
                                              "range_end",
                                              "entity",
                                              "slot_name")

        fun toSlot() = Slot(rawValue = raw_value!!,
                            value = value!!.toSlotValue(),
                            range = if (range_start != -1) Range(range_start!!, range_end!!) else null,
                            entity = entity!!,
                            slotName = slot_name!!)
    }

    class CTaggedEntities(p: Pointer) : Structure(p), Structure.ByReference {
        init {
            read()
        }

        @JvmField var entities: Pointer? = null
        @JvmField var size: Int? = null

        override fun getFieldOrder() = listOf("entities", "size")

        fun toTaggedEntityList(): List<TaggedEntity> =
                if (size != null && size!! > 0)
                    CTaggedEntity(entities!!).toArray(size!!).map { (it as CTaggedEntity).toTaggedEntity() }
                else listOf<TaggedEntity>()


    }

    class CTaggedEntity(p: Pointer) : Structure(p), Structure.ByReference {
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

        fun toTaggedEntity() = TaggedEntity(value = value!!,
                                            range = if (range_start != -1) Range(range_start!!, range_end!!) else null,
                                            entity = entity!!,
                                            slotName = slot_name!!)
    }
}
