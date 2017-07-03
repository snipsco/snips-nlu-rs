//
//  NluEngine.swift
//  NluEngine
//
//  Created by Kevin Lefevre on 14/06/2017.
//  Copyright © 2017 Snips. All rights reserved.
//

import Foundation
import Clibsnips_queries

public struct NluEngineError: Error {
    public let message: String
    
    static var getLast: NluEngineError {
        let buffer = UnsafeMutablePointer<UnsafeMutablePointer<Int8>?>.allocate(capacity: 1024)
        nlu_engine_get_last_error(buffer)
        return NluEngineError(message: String(cString: buffer.pointee!))
    }
}

public struct IntentParserResult {
    public let input: String
    public let intent: IntentClassifierResult?
    public let slots: [Slot]
    
    init(cResult: CIntentParserResult) {
        self.input = String(cString: cResult.input)
        
        if let cClassifierResult = cResult.intent?.pointee {
            self.intent = IntentClassifierResult(cResult: cClassifierResult)
        } else {
            self.intent = nil
        }
        
        if let cSlotList = cResult.slots?.pointee {
            self.slots = UnsafeBufferPointer(start: cSlotList.slots, count: Int(cSlotList.size)).map(Slot.init)
        } else {
            self.slots = []
        }
    }
}

public struct IntentClassifierResult {
    public let intentName: String
    public let probability: Float
    
    init(cResult: CIntentClassifierResult) {
        self.intentName = String(cString: cResult.intent_name)
        self.probability = cResult.probability
    }
}

public struct Slot {
    public let value: String
    public let range: Range<Int>
    public let entity: String
    public let slotName: String
    
    init(cSlot: CSlot) {
        self.value = String(cString: cSlot.value)
        self.range = Range(uncheckedBounds: (Int(cSlot.range_start), Int(cSlot.range_end)))
        self.entity = String(cString: cSlot.entity)
        self.slotName = String(cString: cSlot.slot_name)
    }
}

public class NluEngine {
    private var client: OpaquePointer? = nil
    
    public init(assistantDirectoryURL: URL) throws {
        guard nlu_engine_create_from_dir(assistantDirectoryURL.absoluteString, &client) == OK else { throw NluEngineError.getLast }
    }
    
    public init(assistantZipFile: Data) throws {
        try assistantZipFile.withUnsafeBytes { (bytes: UnsafePointer<UInt8>) in
            guard nlu_engine_create_from_binary(bytes, UInt32(assistantZipFile.count), &client) == OK else { throw NluEngineError.getLast }
        }
    }
    
    deinit {
        if client != nil {
            nlu_engine_destroy_client(client)
            client = nil
        }
    }
    
    public func parse(string: String, filter: [String]) throws -> IntentParserResult {
        var cResult: UnsafeMutablePointer<CIntentParserResult>? = nil;
        guard nlu_engine_run_parse(self.client, string, &cResult) == OK else { throw NluEngineError.getLast }
        defer { nlu_engine_destroy_result(cResult) }
        guard let result = cResult?.pointee else { throw NluEngineError(message: "Can't retrieve result")}
        return IntentParserResult(cResult: result)
    }
}
