//
//  NluEngine.swift
//  NluEngine
//
//  Created by Kevin Lefevre on 14/06/2017.
//  Copyright © 2017 Snips. All rights reserved.
//

import Foundation
import Clibsnips_nlu

public struct NluEngineError: Error {
    public let message: String

    static var getLast: NluEngineError {
        let buffer = UnsafeMutablePointer<UnsafePointer<Int8>?>.allocate(capacity: 1024)
        snips_nlu_engine_get_last_error(buffer)
        return NluEngineError(message: String(cString: buffer.pointee!))
    }
}

public struct IntentParserResult {
    public let input: String
    public let intent: IntentClassifierResult
    public let slots: [Slot]

    init(cResult: CIntentParserResult) throws {
        self.input = String(cString: cResult.input)
        self.intent = IntentClassifierResult(cResult: cResult.intent.pointee)

        let cSlotList = cResult.slots.pointee
        self.slots = try UnsafeBufferPointer(start: cSlotList.slots, count: Int(cSlotList.size)).map(Slot.init)
    }
    
    init(input: String, intent: IntentClassifierResult, slots: [Slot]) {
        self.input = input
        self.intent = intent
        self.slots = slots
    }
}

extension IntentParserResult: Equatable {}

public struct IntentClassifierResult {
    public let intentName: String?
    public let confidenceScore: Float

    init(cResult: CIntentClassifierResult) {
        if let cIntentName = cResult.intent_name {
            self.intentName = String(cString: cIntentName)
        } else {
            self.intentName = nil
        }
        self.confidenceScore = cResult.confidence_score
    }
    
    init(intentName: String?, confidenceScore: Float) {
        self.intentName = intentName
        self.confidenceScore = confidenceScore
    }
}

extension IntentClassifierResult: Equatable {}

public enum SlotValue {
    case custom(String)
    case number(NumberValue)
    case ordinal(OrdinalValue)
    case instantTime(InstantTimeValue)
    case timeInterval(TimeIntervalValue)
    case amountOfMoney(AmountOfMoneyValue)
    case temperature(TemperatureValue)
    case duration(DurationValue)
    case percentage(PercentageValue)
    case musicAlbum(String)
    case musicArtist(String)
    case musicTrack(String)

    init(cSlotValue: CSlotValue) throws {
        switch cSlotValue.value_type {
        case SNIPS_SLOT_VALUE_TYPE_CUSTOM:
            let x = cSlotValue.value.assumingMemoryBound(to: CChar.self)
            self = .custom(String(cString: x))

        case SNIPS_SLOT_VALUE_TYPE_NUMBER:
            let x = cSlotValue.value.assumingMemoryBound(to: CDouble.self)
            self = .number(x.pointee)

        case SNIPS_SLOT_VALUE_TYPE_ORDINAL:
            let x = cSlotValue.value.assumingMemoryBound(to: CInt.self)
            self = .ordinal(OrdinalValue(x.pointee))

        case SNIPS_SLOT_VALUE_TYPE_INSTANTTIME:
            let x = cSlotValue.value.assumingMemoryBound(to:  CInstantTimeValue.self)
            self = .instantTime(try InstantTimeValue(cValue: x.pointee))

        case SNIPS_SLOT_VALUE_TYPE_TIMEINTERVAL:
            let x = cSlotValue.value.assumingMemoryBound(to: CTimeIntervalValue.self)
            self = .timeInterval(TimeIntervalValue(cValue: x.pointee))

        case SNIPS_SLOT_VALUE_TYPE_AMOUNTOFMONEY:
            let x = cSlotValue.value.assumingMemoryBound(to: CAmountOfMoneyValue.self)
            self = .amountOfMoney(try AmountOfMoneyValue(cValue: x.pointee))

        case SNIPS_SLOT_VALUE_TYPE_TEMPERATURE:
            let x = cSlotValue.value.assumingMemoryBound(to: CTemperatureValue.self)
            self = .temperature(TemperatureValue(cValue: x.pointee))

        case SNIPS_SLOT_VALUE_TYPE_DURATION:
            let x = cSlotValue.value.assumingMemoryBound(to: CDurationValue.self)
            self = .duration(try DurationValue(cValue: x.pointee))

        case SNIPS_SLOT_VALUE_TYPE_PERCENTAGE:
            let x = cSlotValue.value.assumingMemoryBound(to: CDouble.self)
            self = .percentage(x.pointee)

        case SNIPS_SLOT_VALUE_TYPE_MUSICALBUM:
            let x = cSlotValue.value.assumingMemoryBound(to: CChar.self)
            self = .musicAlbum(String(cString: x))

        case SNIPS_SLOT_VALUE_TYPE_MUSICARTIST:
            let x = cSlotValue.value.assumingMemoryBound(to: CChar.self)
            self = .musicArtist(String(cString: x))

        case SNIPS_SLOT_VALUE_TYPE_MUSICTRACK:
            let x = cSlotValue.value.assumingMemoryBound(to: CChar.self)
            self = .musicTrack(String(cString: x))

        default: throw NluEngineError(message: "Internal error: Bad type conversion")
        }
    }
}

extension SlotValue: Equatable {}

public typealias NumberValue = Double

public typealias PercentageValue = Double

public typealias OrdinalValue = Int

public struct InstantTimeValue {
    public let value: String
    public let grain: Grain
    public let precision: Precision

    init(cValue: CInstantTimeValue) throws {
        self.value = String(cString: cValue.value)
        self.grain = try Grain(cValue: cValue.grain)
        self.precision = try Precision(cValue: cValue.precision)
    }
    
    init(value: String, grain: Grain, precision: Precision) {
        self.value = value
        self.grain = grain
        self.precision = precision
    }
}

extension InstantTimeValue: Equatable {}

public struct TimeIntervalValue {
    public let from: String?
    public let to: String?

    init(cValue: CTimeIntervalValue) {
        if let cFrom = cValue.from {
            self.from = String(cString: cFrom)
        } else {
            self.from = nil
        }
        if let cTo = cValue.from {
            self.to = String(cString: cTo)
        } else {
            self.to = nil
        }
    }
    
    init(from: String?, to: String?) {
        self.from = from
        self.to = to
    }
}

extension TimeIntervalValue: Equatable {}

public struct AmountOfMoneyValue {
     public let value: Float
     public let precision: Precision
     public let unit: String?

    init(cValue: CAmountOfMoneyValue) throws {
        self.value = cValue.value
        self.precision = try Precision(cValue: cValue.precision)
        if let cUnit = cValue.unit {
            self.unit = String(cString: cUnit)
        } else {
            self.unit = nil
        }
    }
    
    init(value: Float, precision: Precision, unit: String?) {
        self.value = value
        self.precision = precision
        self.unit = unit
    }
}

extension AmountOfMoneyValue: Equatable {}

public struct TemperatureValue {
     public let value: Float
     public let unit: String?

    init(cValue: CTemperatureValue) {
        self.value = cValue.value
        if let cUnit = cValue.unit {
            self.unit = String(cString: cUnit)
        } else {
            self.unit = nil
        }
    }
    
    init(value: Float, unit: String?) {
        self.value = value
        self.unit = unit
    }
}

extension TemperatureValue: Equatable {}

public struct DurationValue {
     public let years: Int
     public let quarters: Int
     public let months: Int
     public let weeks: Int
     public let days: Int
     public let hours: Int
     public let minutes: Int
     public let seconds: Int
     public let precision: Precision

    init(cValue: CDurationValue) throws {
        self.years = Int(truncatingIfNeeded: cValue.years)
        self.quarters = Int(truncatingIfNeeded: cValue.quarters)
        self.months = Int(truncatingIfNeeded: cValue.months)
        self.weeks = Int(truncatingIfNeeded: cValue.weeks)
        self.days = Int(truncatingIfNeeded: cValue.days)
        self.hours = Int(truncatingIfNeeded: cValue.hours)
        self.minutes = Int(truncatingIfNeeded: cValue.minutes)
        self.seconds = Int(truncatingIfNeeded: cValue.seconds)
        self.precision = try Precision(cValue: cValue.precision)
    }
    
    init(years: Int, quarters: Int, months: Int, weeks: Int, days: Int, hours: Int, minutes: Int, seconds: Int, precision: Precision) {
        self.years = years
        self.quarters = quarters
        self.months = months
        self.weeks = weeks
        self.days = days
        self.hours = hours
        self.minutes = minutes
        self.seconds = seconds
        self.precision = precision
    }
}

extension DurationValue: Equatable {}

public enum Grain {
    case year
    case quarter
    case month
    case week
    case day
    case hour
    case minute
    case second

    init(cValue: SNIPS_GRAIN) throws {
        switch cValue {
        case SNIPS_GRAIN_YEAR: self = .year
        case SNIPS_GRAIN_QUARTER: self = .quarter
        case SNIPS_GRAIN_MONTH: self = .month
        case SNIPS_GRAIN_WEEK: self = .week
        case SNIPS_GRAIN_DAY: self = .day
        case SNIPS_GRAIN_HOUR: self = .hour
        case SNIPS_GRAIN_MINUTE: self = .minute
        case SNIPS_GRAIN_SECOND: self = .second
        default: throw NluEngineError(message: "Internal error: Bad type conversion")
        }
    }
}

public enum Precision {
    case approximate
    case exact

    init(cValue: SNIPS_PRECISION) throws {
        switch cValue {
        case SNIPS_PRECISION_APPROXIMATE: self = .approximate
        case SNIPS_PRECISION_EXACT: self = .exact
        default: throw NluEngineError(message: "Internal error: Bad type conversion")
        }
    }
}

public struct Slot: Equatable {
    public let rawValue: String
    public let value: SlotValue
    public let range: Range<Int>
    public let entity: String
    public let slotName: String
    public let confidenceScore: Float?

    init(cSlot: CSlot) throws {
        self.rawValue = String(cString: cSlot.raw_value)
        self.value = try SlotValue(cSlotValue: cSlot.value)
        self.range = Range(uncheckedBounds: (Int(cSlot.range_start), Int(cSlot.range_end)))
        self.entity = String(cString: cSlot.entity)
        self.slotName = String(cString: cSlot.slot_name)
        self.confidenceScore = cSlot.confidence_score >= 0 ? cSlot.confidence_score : nil
    }
    
    init(rawValue: String, value: SlotValue, range: Range<Int>, entity: String, slotName: String, confidenceScore: Float? = nil) {
        self.rawValue = rawValue
        self.value = value
        self.range = range
        self.entity = entity
        self.slotName = slotName
        self.confidenceScore = confidenceScore
    }
}

extension CStringArray {
    init(array: [String]) {
        let data = UnsafeMutablePointer<UnsafePointer<Int8>?>.allocate(capacity: array.count)
        array.enumerated().forEach {
            data.advanced(by: $0).pointee = UnsafePointer(strdup($1))
        }
        
        self.init()
        self.data = UnsafePointer(data)
        self.size = Int32(array.count)
    }
    
    func destroy() {
        let mutating = UnsafeMutablePointer(mutating: data)
        for idx in 0..<size {
            free(UnsafeMutableRawPointer(mutating: data.advanced(by: Int(idx)).pointee))
        }
        mutating?.deallocate()
    }
}

public class NluEngine {
    private var client: OpaquePointer? = nil

    /**
     Loads an NluEngine from a directory
     */
    public init(nluEngineDirectoryURL: URL) throws {
        guard snips_nlu_engine_create_from_dir(nluEngineDirectoryURL.path, &client) == SNIPS_RESULT_OK else {
            throw NluEngineError.getLast
        }
    }

    /**
     Loads an NluEngine from zipped data
     
     - Parameter nluEngineZipData: Binary data corresponding to a zipped NluEngine instance
     */
    public init(nluEngineZipData: Data) throws {
        try nluEngineZipData.withUnsafeBytes { (bytes: UnsafeRawBufferPointer) in
            guard snips_nlu_engine_create_from_zip(bytes.baseAddress?.assumingMemoryBound(to: UInt8.self), UInt32(nluEngineZipData.count), &client) == SNIPS_RESULT_OK else {
                throw NluEngineError.getLast
            }
        }
    }

    deinit {
        if client != nil {
            snips_nlu_engine_destroy_client(client)
            client = nil
        }
    }

    /**
     Extracts intent and slots from the input
     
     - Parameter string: input to process
     - Parameter intentsWhitelist: optional list of intents used to restrict the parsing scope
     - Parameter intentsBlacklist: optional list of intents to exclude during parsing
     */
    public func parse(string: String, intentsWhitelist: [String]? = nil, intentsBlacklist: [String]? = nil) throws -> IntentParserResult {
        var cResult: UnsafePointer<CIntentParserResult>? = nil;
        var whiteListArray: CStringArray?
        var blackListArray: CStringArray?
        defer {
            snips_nlu_engine_destroy_result(UnsafeMutablePointer(mutating: cResult))
            whiteListArray?.destroy()
            blackListArray?.destroy()
        }
        var whitelist: UnsafePointer<CStringArray>?
        var blacklist: UnsafePointer<CStringArray>?
        
        if let unwrappedIntentsWhitelist = intentsWhitelist {
            whiteListArray = CStringArray(array: unwrappedIntentsWhitelist)
            whitelist = withUnsafePointer(to: &whiteListArray!) { $0 }
        }
        
        if let unwrappedIntentsBlacklist = intentsBlacklist {
            blackListArray = CStringArray(array: unwrappedIntentsBlacklist)
            blacklist = withUnsafePointer(to: &blackListArray!) { $0 }
        }
        
        guard snips_nlu_engine_run_parse(self.client, string, whitelist, blacklist, &cResult) == SNIPS_RESULT_OK else {
            throw NluEngineError.getLast
        }

        guard let result = cResult?.pointee else { throw NluEngineError(message: "Can't retrieve result")}
        return try IntentParserResult(cResult: result)
    }
    
    /**
     
     Extracts slots from the input when the intent is known
     
     - Parameter string: input to process
     - Parameter intent: intent which the input corresponds to
     */
    public func getSlots(string: String, intent: String) throws -> [Slot] {
        var cSlots: UnsafePointer<CSlotList>? = nil;
        defer {
            snips_nlu_engine_destroy_slots(UnsafeMutablePointer(mutating: cSlots))
        }
        
        guard snips_nlu_engine_run_get_slots(self.client, string, intent, &cSlots) == SNIPS_RESULT_OK else {
            throw NluEngineError.getLast
        }
        
        guard let cSlotList = cSlots?.pointee else { throw NluEngineError(message: "Can't retrieve result")}
        return try UnsafeBufferPointer(start: cSlotList.slots, count: Int(cSlotList.size)).map(Slot.init)
    }
    
    /**
     Extract the list of intents ranked by their confidence score
     */
    public func getIntents(string: String) throws -> [IntentClassifierResult] {
        var cResults: UnsafePointer<CIntentClassifierResultList>? = nil;
        defer {
            snips_nlu_engine_destroy_intent_classifier_results(UnsafeMutablePointer(mutating: cResults))
        }
        
        guard snips_nlu_engine_run_get_intents(self.client, string, &cResults) == SNIPS_RESULT_OK else {
            throw NluEngineError.getLast
        }
        
        guard let cResultList = cResults?.pointee else { throw NluEngineError(message: "Can't retrieve result")}
        return UnsafeBufferPointer(start: cResultList.intent_classifier_results, count: Int(cResultList.size)).map(IntentClassifierResult.init)
    }
}
