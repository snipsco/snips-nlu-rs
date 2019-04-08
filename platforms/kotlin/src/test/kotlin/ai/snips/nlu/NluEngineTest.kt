package ai.snips.nlu

import com.google.common.truth.Truth.assertThat
import org.junit.Test
import java.io.File


class NluEngineTest {

    @Test
    fun modelVersionWorks() {
        assertThat(NluEngine.modelVersion()).isNotEmpty()
    }

    @Test
    fun createFromDirWorks() {
        NluEngine(File("../../data/tests/models/nlu_engine")).use {
            it.parse("make me two cups of hot tea").apply {
                assertThat(input).isEqualTo("make me two cups of hot tea")
                assertThat(intent.intentName!!).isEqualTo("MakeTea")
                assertThat(slots).hasSize(2)
                assertThat(slots.map { it.slotName }).containsAllOf("beverage_temperature", "number_of_cups")
            }
        }
    }

    @Test
    fun createFromZipWorks() {
        NluEngine(File("../../data/tests/models/nlu_engine.zip").readBytes()).use {
            it.parse("make me two cups of hot tea").apply {
                assertThat(input).isEqualTo("make me two cups of hot tea")
                assertThat(intent.intentName!!).isEqualTo("MakeTea")
                assertThat(slots).hasSize(2)
                assertThat(slots.map { it.slotName }).containsAllOf("beverage_temperature", "number_of_cups")
            }
        }
    }

    @Test
    fun parseWithWhitelistWorks() {
        NluEngine(File("../../data/tests/models/nlu_engine")).use {
            it.parse("make me two cups of hot tea", listOf("MakeCoffee"), null).apply {
                assertThat(input).isEqualTo("make me two cups of hot tea")
                assertThat(intent.intentName!!).isEqualTo("MakeCoffee")
            }
        }
    }

    @Test
    fun parseWithBlacklistWorks() {
        NluEngine(File("../../data/tests/models/nlu_engine")).use {
            it.parse("make me two cups of hot tea", null, listOf("MakeTea")).apply {
                assertThat(input).isEqualTo("make me two cups of hot tea")
                assertThat(intent.intentName!!).isEqualTo("MakeCoffee")
            }
        }
    }

    @Test
    fun parseIntoJsonWorks() {
        NluEngine(File("../../data/tests/models/nlu_engine")).use {
            it.parseIntoJson("make me two cups of hot tea").apply {
                assertThat(this).isNotNull()
                assertThat(this).contains("make me two cups of hot tea")
                assertThat(this).contains("MakeTea")
                assertThat(this).contains("beverage_temperature")
                assertThat(this).contains("number_of_cups")
            }
        }
    }

    @Test
    fun parseIntoJsonWithWhitelistWorks() {
        NluEngine(File("../../data/tests/models/nlu_engine")).use {
            it.parseIntoJson("make me two cups of hot tea", listOf("MakeCoffee"), null).apply {
                assertThat(this).isNotNull()
                assertThat(this).contains("make me two cups of hot tea")
                assertThat(this).contains("MakeCoffee")
            }
        }
    }

    @Test
    fun parseIntoJsonWithBlakclistWorks() {
        NluEngine(File("../../data/tests/models/nlu_engine")).use {
            it.parseIntoJson("make me two cups of hot tea", null, listOf("MakeTea")).apply {
                assertThat(this).isNotNull()
                assertThat(this).contains("make me two cups of hot tea")
                assertThat(this).contains("MakeCoffee")
            }
        }
    }

    @Test
    fun getSlotsWorks() {
        NluEngine(File("../../data/tests/models/nlu_engine")).use {
            it.getSlots("make me two cups of hot tea", "MakeTea").apply {
                assertThat(this).hasSize(2)
                assertThat(this.map { it.slotName }).containsAllOf("beverage_temperature", "number_of_cups")
            }
        }
    }

    @Test
    fun getSlotsIntoJsonWorks() {
        NluEngine(File("../../data/tests/models/nlu_engine")).use {
            it.getSlotsIntoJson("make me two cups of hot tea", "MakeTea").apply {
                assertThat(this).isNotNull()
                assertThat(this).contains("beverage_temperature")
                assertThat(this).contains("number_of_cups")
            }
        }
    }

    @Test
    fun getIntentsWorks() {
        NluEngine(File("../../data/tests/models/nlu_engine")).use {
            it.getIntents("make me two cups of hot tea").apply {
                assertThat(this).hasSize(3)
                assertThat(this.map { it.intentName })
                        .isEqualTo(listOf("MakeTea", "MakeCoffee", null))
            }
        }
    }

    @Test
    fun getIntentsIntoJsonWorks() {
        NluEngine(File("../../data/tests/models/nlu_engine")).use {
            it.getIntentsIntoJson("make me two cups of hot tea").apply {
                assertThat(this).isNotNull()
                assertThat(this).contains("MakeTea")
                assertThat(this).contains("MakeCoffee")
                assertThat(this).contains("null")
            }
        }
    }

    @Test
    fun funkyCharsArePreserved() {
        NluEngine(File("../../data/tests/models/nlu_engine")).use {
            it.parse("&€£ôœþかたな刀☺ ̿ ̿ ̿'̿'\\̵͇̿̿\\з=(•_•)=ε/̵͇̿̿/'̿'̿ ̿").apply {
                assertThat(input).isEqualTo("&€£ôœþかたな刀☺ ̿ ̿ ̿'̿'\\̵͇̿̿\\з=(•_•)=ε/̵͇̿̿/'̿'̿ ̿")
            }
        }
    }
}
