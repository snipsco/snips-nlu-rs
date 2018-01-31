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

    func testCreationFromZipfile() {
        let fileURL = Bundle(for: type(of: self)).url(forResource: "sample_config", withExtension: "zip")!
        let data = try! Data(contentsOf: fileURL)

        let nluEngine = try? NluEngine(assistantZipFile: data)

        XCTAssertNotNil(nluEngine)
    }

    func testCreationFromDirectory() {
        let directoryURL = Bundle(for: type(of: self)).url(forResource: "configurations", withExtension: nil)!

        let nluEngine = try? NluEngine(assistantDirectoryURL: directoryURL)

        XCTAssertNotNil(nluEngine)
    }
}
