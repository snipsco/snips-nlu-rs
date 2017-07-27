package ai.snips.queries

import ai.snips.queries.SlotValue.AmountOfMoneyValue
import ai.snips.queries.SlotValue.CustomValue
import ai.snips.queries.SlotValue.DurationValue
import ai.snips.queries.SlotValue.InstantTimeValue
import ai.snips.queries.SlotValue.NumberValue
import ai.snips.queries.SlotValue.OrdinalValue
import ai.snips.queries.SlotValue.TemperatureValue
import ai.snips.queries.SlotValue.TimeIntervalValue
import ai.snips.queries.SlotValue.Type.AMOUNT_OF_MONEY
import ai.snips.queries.SlotValue.Type.CUSTOM
import ai.snips.queries.SlotValue.Type.DURATION
import ai.snips.queries.SlotValue.Type.INSTANT_TIME
import ai.snips.queries.SlotValue.Type.NUMBER
import ai.snips.queries.SlotValue.Type.ORDINAL
import ai.snips.queries.SlotValue.Type.TEMPERATURE
import ai.snips.queries.SlotValue.Type.TIME_INTERVAL
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure
import com.sun.jna.ptr.PointerByReference
import com.sun.jna.toJnaPointer
import java.io.Closeable
import java.io.File
import kotlin.system.measureTimeMillis
import ai.snips.queries.NluEngine.SnipsQueriesClientLibrary.Companion.INSTANCE as LIB

data class Range(val start: Int, val end: Int)

data class Slot(val rawValue: String, val value: SlotValue, val range: Range?, val entity: String, val slotName: String)

enum class Precision {APPROXIMATE, EXACT }
enum class Grain { YEAR, QUARTER, MONTH, WEEK, DAY, HOUR, MINUTE, SECOND }

// TODO : add converters to JSR310 / ThreeTen types
sealed class SlotValue(val type: SlotValue.Type) {

    enum class Type {
        CUSTOM,
        NUMBER,
        ORDINAL,
        INSTANT_TIME,
        TIME_INTERVAL,
        AMOUNT_OF_MONEY,
        TEMPERATURE,
        DURATION,
    }

    data class CustomValue(val value: String) : SlotValue(CUSTOM)
    data class NumberValue(val value: Double) : SlotValue(NUMBER)
    data class OrdinalValue(val value: Long) : SlotValue(ORDINAL)
    data class InstantTimeValue(val value: String, val grain: Grain, val precision: Precision) : SlotValue(INSTANT_TIME)
    data class TimeIntervalValue(val from: String, val to: String) : SlotValue(TIME_INTERVAL)
    data class AmountOfMoneyValue(val value: Float, val precision: Precision, val unit: String) : SlotValue(AMOUNT_OF_MONEY)
    data class TemperatureValue(val value: Float, val unit: String) : SlotValue(TEMPERATURE)
    data class DurationValue(val years: Long,
                             val quarters: Long,
                             val months: Long,
                             val weeks: Long,
                             val days: Long,
                             val hours: Long,
                             val minutes: Long,
                             val seconds: Long,
                             val precision: Precision) : SlotValue(DURATION)
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

        const val RUST_ENCODING = "utf-8"

        fun String.toPointer(): Pointer = this.toJnaPointer(ai.snips.queries.NluEngine.RUST_ENCODING)
        fun Pointer.readString(): String = this.getString(0, ai.snips.queries.NluEngine.RUST_ENCODING)

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

    fun tag(input: String, intent: String): List<TaggedEntity> =
            CTaggedEntities(PointerByReference().apply {
                parseError(LIB.nlu_engine_run_tag(client, input.toPointer(), intent.toPointer(), this))
            }.value).let {
                it.toTaggedEntityList().apply {
                    // we don't want jna to try and sync this struct after the call as we're destroying it
                    // /!\ removing that will make the app crash semi randomly...
                    it.autoRead = false
                    LIB.nlu_engine_destroy_tagged_entity_list(it)
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
        fun nlu_engine_run_tag(client: Pointer, input: Pointer, intent: Pointer, result: PointerByReference): Int
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

        @JvmField var input: Pointer? = null
        @JvmField var intent: CIntentClassifierResult? = null
        @JvmField var slots: CSlots? = null

        override fun getFieldOrder() = listOf("input",
                                              "intent",
                                              "slots")

        fun toIntentParserResult() = IntentParserResult(input = input!!.readString(),
                                                        intent = intent?.toIntentClassifierResult(),
                                                        slots = slots?.toSlotList() ?: listOf())

    }

    class CIntentClassifierResult : Structure(), Structure.ByReference {
        @JvmField var intent_name: Pointer? = null
        @JvmField var probability: Float? = null

        override fun getFieldOrder() = listOf("intent_name", "probability")

        fun toIntentClassifierResult() = IntentClassifierResult(intentName = intent_name!!.readString(), probability = probability!!)
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
            CUSTOM -> CustomValue(value!!.readString())
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

        @JvmField var value: Pointer? = null
        @JvmField var grain: Int? = null
        @JvmField var precision: Int? = null
        override fun getFieldOrder() = listOf("value", "grain", "precision")
        fun toInstantTimeValue(): InstantTimeValue {
            return InstantTimeValue(value = value!!.readString(),
                                    grain = CGrain.toGrain(grain!!),
                                    precision = CPrecision.toPrecision(precision!!))

        }
    }

    class CTimeIntervalValue(p: Pointer) : Structure(p), Structure.ByReference {
        init {
            read()
        }

        @JvmField var from: Pointer? = null
        @JvmField var to: Pointer? = null
        override fun getFieldOrder() = listOf("from", "to")
        fun toTimeIntervalValue() = TimeIntervalValue(from = from!!.readString(), to = to!!.readString())
    }

    class CAmountOfMoneyValue(p: Pointer) : Structure(p), Structure.ByReference {
        init {
            read()
        }

        @JvmField var value: Float? = null
        @JvmField var precision: Int? = null
        @JvmField var unit: Pointer? = null
        override fun getFieldOrder() = listOf("value", "precision", "unit")
        fun toAmountOfMoneyValue() = AmountOfMoneyValue(value = value!!,
                                                        precision = CPrecision.toPrecision(precision!!),
                                                        unit = unit!!.readString())
    }

    class CTemperatureValue(p: Pointer) : Structure(p), Structure.ByReference {
        init {
            read()
        }

        @JvmField var value: Float? = null
        @JvmField var unit: Pointer? = null
        override fun getFieldOrder() = listOf("value", "unit")
        fun toTemperatureValue() = TemperatureValue(value = value!!,
                                                    unit = unit!!.readString())

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

        @JvmField var raw_value: Pointer? = null
        @JvmField var value: CSlotValue? = null
        @JvmField var range_start: Int? = null
        @JvmField var range_end: Int? = null
        @JvmField var entity: Pointer? = null
        @JvmField var slot_name: Pointer? = null

        override fun getFieldOrder() = listOf("raw_value",
                                              "value",
                                              "range_start",
                                              "range_end",
                                              "entity",
                                              "slot_name")

        fun toSlot() = Slot(rawValue = raw_value!!.readString(),
                            value = value!!.toSlotValue(),
                            range = if (range_start != -1) Range(range_start!!, range_end!!) else null,
                            entity = entity!!.readString(),
                            slotName = slot_name!!.readString())
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

        @JvmField var value: Pointer? = null
        @JvmField var range_start: Int? = null
        @JvmField var range_end: Int? = null
        @JvmField var entity: Pointer? = null
        @JvmField var slot_name: Pointer? = null

        override fun getFieldOrder() = listOf("value",
                                              "range_start",
                                              "range_end",
                                              "entity",
                                              "slot_name")

        fun toTaggedEntity() = TaggedEntity(value = value!!.readString(),
                                            range = if (range_start != -1) Range(range_start!!, range_end!!) else null,
                                            entity = entity!!.readString(),
                                            slotName = slot_name!!.readString())
    }
}
