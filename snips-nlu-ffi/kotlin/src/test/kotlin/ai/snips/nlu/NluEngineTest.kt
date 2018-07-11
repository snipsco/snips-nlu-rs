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
        NluEngine(File("../../data/tests/models/trained_engine")).use {
            it.parse("make me two cups of hot tea").apply {
                assertThat(input).isEqualTo("make me two cups of hot tea")
                assertThat(intent).isNotNull()
                assertThat(intent!!.intentName).isEqualTo("MakeTea")
                assertThat(slots).hasSize(2)
                assertThat(slots.map { it.slotName }).containsAllOf("beverage_temperature", "number_of_cups")
            }
        }
    }

    @Test
    fun parseIntoJsonWorks() {
        NluEngine(File("../../data/tests/configurations")).use {
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
    fun funkyCharsArePreserved() {
        NluEngine(File("../../data/tests/configurations")).use {
            it.parse("&€£ôœþかたな刀☺ ̿ ̿ ̿'̿'\\̵͇̿̿\\з=(•_•)=ε/̵͇̿̿/'̿'̿ ̿").apply {
                assertThat(input).isEqualTo("&€£ôœþかたな刀☺ ̿ ̿ ̿'̿'\\̵͇̿̿\\з=(•_•)=ε/̵͇̿̿/'̿'̿ ̿")
            }
        }
    }
}
