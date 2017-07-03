//
//  NluEngineTests.swift
//  NluEngineTests
//
//  Created by Kevin Lefevre on 16/06/2017.
//  Copyright © 2017 Snips. All rights reserved.
//

import XCTest
@testable import SnipsQueries

class NluEngineTests: XCTestCase {
    
    override func setUp() {
        super.setUp()
        
    }
    
    override func tearDown() {
        // Put teardown code here. This method is called after the invocation of each test method in the class.
        super.tearDown()
    }
    
    func testCreationFromZipfile() {
        let fileURL = Bundle(for: type(of: self)).url(forResource: "sample_assistant", withExtension: "zip")!
        let data = try! Data(contentsOf: fileURL)
        
        let nluEngine = try? NluEngine(assistantZipFile: data)
        
        XCTAssertNotNil(nluEngine)
    }
    
    func testCreationFromDirectory() {
        // TODO: Create directory with an assistant inside and check if everything works
    }
}
