//
//  NluEngineTests.swift
//  NluEngineTests
//
//  Created by Kevin Lefevre on 16/06/2017.
//  Copyright © 2017 Snips. All rights reserved.
//

import XCTest
@testable import SnipsNlu

class NluEngineTests: XCTestCase {
    func testCreationFromDirectory() {
        let directoryURL = Bundle(for: type(of: self)).url(forResource: "nlu_engine", withExtension: nil)!

        let nluEngine = try? NluEngine(nluEngineDirectoryURL: directoryURL)

        XCTAssertNotNil(nluEngine)
    }

    func testCreationFromZip() {
        let fileURL = Bundle(for: type(of: self)).url(forResource: "nlu_engine", withExtension: "zip")!
        let data = try! Data(contentsOf: fileURL)

        let nluEngine = try? NluEngine(nluEngineZipData: data)

        XCTAssertNotNil(nluEngine)
    }

    func testParse() {
        let directoryURL = Bundle(for: type(of: self)).url(forResource: "nlu_engine", withExtension: nil)!

        let nluEngine = try! NluEngine(nluEngineDirectoryURL: directoryURL)

        let result = try! nluEngine.parse(string: "Make me two cups of coffee please")
        let expectedSlot = Slot(rawValue: "two", value: SlotValue.number(2.0), range: 8..<11, entity: "snips/number", slotName: "number_of_cups")
        XCTAssertEqual("MakeCoffee", result.intent.intentName)
        XCTAssertEqual([expectedSlot], result.slots)
    }
    
    func testParseWithWhitelist() {
        let directoryURL = Bundle(for: type(of: self)).url(forResource: "nlu_engine", withExtension: nil)!

        let nluEngine = try! NluEngine(nluEngineDirectoryURL: directoryURL)

        let result = try! nluEngine.parse(string: "Make me two cups of coffee please", intentsWhitelist: ["MakeTea"])
        let expectedSlot = Slot(rawValue: "two", value: SlotValue.number(2.0), range: 8..<11, entity: "snips/number", slotName: "number_of_cups")
        XCTAssertEqual("MakeTea", result.intent.intentName)
        XCTAssertEqual([expectedSlot], result.slots)
    }
    
    func testParseWithBlacklist() {
        let directoryURL = Bundle(for: type(of: self)).url(forResource: "nlu_engine", withExtension: nil)!
        
        let nluEngine = try! NluEngine(nluEngineDirectoryURL: directoryURL)
        
        let result = try! nluEngine.parse(string: "Make me two cups of coffee please", intentsBlacklist: ["MakeCoffee"])
        let expectedSlot = Slot(rawValue: "two", value: SlotValue.number(2.0), range: 8..<11, entity: "snips/number", slotName: "number_of_cups")
        XCTAssertEqual("MakeTea", result.intent.intentName)
        XCTAssertEqual([expectedSlot], result.slots)
    }
    
    func testGetSlots() {
        let directoryURL = Bundle(for: type(of: self)).url(forResource: "nlu_engine", withExtension: nil)!
        
        let nluEngine = try! NluEngine(nluEngineDirectoryURL: directoryURL)
        
        let slots = try! nluEngine.getSlots(string: "Make me two cups of coffee please", intent: "MakeCoffee")
        let expectedSlot = Slot(rawValue: "two", value: SlotValue.number(2.0), range: 8..<11, entity: "snips/number", slotName: "number_of_cups")
        XCTAssertEqual([expectedSlot], slots)
    }
    
    func testGetIntents() {
        let directoryURL = Bundle(for: type(of: self)).url(forResource: "nlu_engine", withExtension: nil)!
        
        let nluEngine = try! NluEngine(nluEngineDirectoryURL: directoryURL)
        
        let intentsResults = try! nluEngine.getIntents(string: "Make me two cups of coffee please")
        let intents = intentsResults.map { $0.intentName }
        let expectedIntents = ["MakeCoffee", "MakeTea", nil]
        XCTAssertEqual(expectedIntents, intents)
    }
}
