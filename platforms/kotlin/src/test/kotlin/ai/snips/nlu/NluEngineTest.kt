package ai.snips.nlu

import ai.snips.nlu.ontology.SlotValue.CustomValue
import ai.snips.nlu.ontology.SlotValue.NumberValue
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
        NluEngine(File("../../data/tests/models/nlu_engine_beverage")).use {
            it.parse("make me two cups of hot tea").apply {
                assertThat(input).isEqualTo("make me two cups of hot tea")
                assertThat(intent.intentName).isEqualTo("MakeTea")
                assertThat(slots).hasSize(2)
                assertThat(slots.map { it.slotName }).containsAllOf("beverage_temperature", "number_of_cups")
            }
        }
    }

    @Test
    fun createFromZipWorks() {
        NluEngine(File("../../data/tests/models/nlu_engine_beverage.zip").readBytes()).use {
            it.parse("make me two cups of hot tea").apply {
                assertThat(input).isEqualTo("make me two cups of hot tea")
                assertThat(intent.intentName).isEqualTo("MakeTea")
                assertThat(slots).hasSize(2)
                assertThat(slots.map { it.slotName }).containsAllOf("beverage_temperature", "number_of_cups")
                assertThat(alternatives).hasSize(0)
            }
        }
    }

    @Test
    fun parseWithWhitelistWorks() {
        NluEngine(File("../../data/tests/models/nlu_engine_beverage")).use {
            it.parse("can you prepare one cup of tea or coffee", listOf("MakeTea"), null).apply {
                assertThat(input).isEqualTo("can you prepare one cup of tea or coffee")
                assertThat(intent.intentName).isEqualTo("MakeTea")
            }
        }
    }

    @Test
    fun parseWithBlacklistWorks() {
        NluEngine(File("../../data/tests/models/nlu_engine_beverage")).use {
            it.parse("can you prepare one cup of tea or coffee", null, listOf("MakeCoffee")).apply {
                assertThat(input).isEqualTo("can you prepare one cup of tea or coffee")
                assertThat(intent.intentName).isEqualTo("MakeTea")
            }
        }
    }

    @Test
    fun parseWithIntentsAlternativesWorks() {
        NluEngine(File("../../data/tests/models/nlu_engine_beverage")).use {
            it.parse("Make me two cups of coffee please", null, null, 1).apply {
                assertThat(input).isEqualTo("Make me two cups of coffee please")
                assertThat(intent.intentName).isEqualTo("MakeCoffee")
                assertThat(alternatives).hasSize(1)
                assertThat(alternatives[0].intent.intentName).isEqualTo("MakeTea")
            }
        }
    }

    @Test
    fun parseWithSlotsAlternativesWorks() {
        NluEngine(File("../../data/tests/models/nlu_engine_game")).use {
            it.parse("I want to play to invader", null, null, 0, 2).apply {
                assertThat(input).isEqualTo("I want to play to invader")
                assertThat(slots.flatMap { it.alternatives }).isEqualTo(listOf(
                        CustomValue(value="Invader War Demo"),
                        CustomValue(value="Space Invader Limited Edition")
                ))
            }
        }
    }

    @Test
    fun parseIntoJsonWorks() {
        NluEngine(File("../../data/tests/models/nlu_engine_beverage")).use {
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
        NluEngine(File("../../data/tests/models/nlu_engine_beverage")).use {
            it.parseIntoJson("can you prepare one cup of tea or coffee", listOf("MakeTea"), null).apply {
                assertThat(this).isNotNull()
                assertThat(this).contains("can you prepare one cup of tea or coffee")
                assertThat(this).contains("MakeTea")
            }
        }
    }

    @Test
    fun parseIntoJsonWithBlacklistWorks() {
        NluEngine(File("../../data/tests/models/nlu_engine_beverage")).use {
            it.parseIntoJson("can you prepare one cup of tea or coffee", null, listOf("MakeCoffee")).apply {
                assertThat(this).isNotNull()
                assertThat(this).contains("can you prepare one cup of tea or coffee")
                assertThat(this).contains("MakeTea")
            }
        }
    }

    @Test
    fun parseIntoJsonWithIntentsAlternativesWorks() {
        NluEngine(File("../../data/tests/models/nlu_engine_beverage")).use {
            it.parseIntoJson("Make me two cups of coffee please", null, null, 1).apply {
                assertThat(this).isNotNull()
                assertThat(this).contains("Make me two cups of coffee please")
                assertThat(this).contains("MakeCoffee")
                assertThat(this).contains("MakeTea")
            }
        }
    }

    @Test
    fun parseIntoJsonWithSlotsAlternativesWorks() {
        NluEngine(File("../../data/tests/models/nlu_engine_game")).use {
            it.parseIntoJson("I want to play to invader", null, null, 0, 2).apply {
                assertThat(this).isNotNull()
                assertThat(this).contains("I want to play to invader")
                assertThat(this).contains("Invader Attack 3")
                assertThat(this).contains("Invader War Demo")
                assertThat(this).contains("Space Invader Limited Edition")
            }
        }
    }

    @Test
    fun getSlotsWorks() {
        NluEngine(File("../../data/tests/models/nlu_engine_beverage")).use {
            it.getSlots("make me two cups of hot tea", "MakeTea").apply {
                assertThat(this).hasSize(2)
                assertThat(this.map { it.slotName }).isEqualTo(listOf(
                        "number_of_cups",
                        "beverage_temperature"
                ))
                assertThat(this.map { it.rawValue }).isEqualTo(listOf(
                        "two",
                        "hot"
                ))
                assertThat(this.map { it.value }).isEqualTo(listOf(
                        NumberValue(value=2.0),
                        CustomValue(value="hot")
                ))
            }
        }
    }

    @Test
    fun getSlotsWithAlternativesWorks() {
        NluEngine(File("../../data/tests/models/nlu_engine_game")).use {
            it.getSlots("I want to play to invader", "PlayGame", 2).apply {
                assertThat(this).hasSize(1)
                assertThat(this[0].slotName).isEqualTo("game")
                assertThat(this[0].rawValue).isEqualTo("invader")
                assertThat(this[0].value).isEqualTo(CustomValue(value="Invader Attack 3"))
                assertThat(this[0].alternatives).isEqualTo(listOf(
                        CustomValue(value="Invader War Demo"),
                        CustomValue(value="Space Invader Limited Edition")
                ))
            }
        }
    }

    @Test
    fun getSlotsIntoJsonWorks() {
        NluEngine(File("../../data/tests/models/nlu_engine_beverage")).use {
            it.getSlotsIntoJson("make me two cups of hot tea", "MakeTea").apply {
                assertThat(this).isNotNull()
                assertThat(this).contains("beverage_temperature")
                assertThat(this).contains("number_of_cups")
            }
        }
    }

    @Test
    fun getSlotsWithAlternativesIntoJsonWorks() {
        NluEngine(File("../../data/tests/models/nlu_engine_game")).use {
            it.getSlotsIntoJson("I want to play to invader", "PlayGame", 2).apply {
                assertThat(this).isNotNull()
                assertThat(this).contains("game")
                assertThat(this).contains("invader")
                assertThat(this).contains("Invader Attack 3")
                assertThat(this).contains("Invader War Demo")
                assertThat(this).contains("Space Invader Limited Edition")
            }
        }
    }

    @Test
    fun getIntentsWorks() {
        NluEngine(File("../../data/tests/models/nlu_engine_beverage")).use {
            it.getIntents("can you prepare one cup of tea or coffee").apply {
                assertThat(this).hasSize(3)
                assertThat(this.map { it.intentName })
                        .isEqualTo(listOf("MakeCoffee", "MakeTea", null))
            }
        }
    }

    @Test
    fun getIntentsIntoJsonWorks() {
        NluEngine(File("../../data/tests/models/nlu_engine_beverage")).use {
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
        NluEngine(File("../../data/tests/models/nlu_engine_beverage")).use {
            it.parse("&€£ôœþかたな刀☺ ̿ ̿ ̿'̿'\\̵͇̿̿\\з=(•_•)=ε/̵͇̿̿/'̿'̿ ̿").apply {
                assertThat(input).isEqualTo("&€£ôœþかたな刀☺ ̿ ̿ ̿'̿'\\̵͇̿̿\\з=(•_•)=ε/̵͇̿̿/'̿'̿ ̿")
            }
        }
    }
}
