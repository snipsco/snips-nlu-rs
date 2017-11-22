package ai.snips.queries

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
        NluEngine(File("../../data/tests/configurations")).use {
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
    fun createFromZipWorks() {
        NluEngine(File("../../data/tests/zip_files/sample_config.zip").readBytes()).use {
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
    fun tagWorks() {
        NluEngine(File("../../data/tests/configurations")).use {
            it.tag("make me two cups of hot tea", "MakeTea").apply {
                assertThat(this).hasSize(2)
                assertThat(this.map { it.slotName }).containsAllOf("beverage_temperature", "number_of_cups")
            }
        }
    }

    @Test
    fun tagWorksOnEmptyResult() {
        NluEngine(File("../../data/tests/configurations")).use {
            println("hello")
            it.tag("Turn off the lights", "MakeTea").apply {
                assertThat(this).hasSize(0)
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
