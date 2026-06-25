import Foundation
import XCTest
@testable import HP41GUI

final class GoldenFlowTests: XCTestCase {
    private struct Fixture: Decodable {
        let schemaVersion: Int
        let cases: [GoldenCase]

        enum CodingKeys: String, CodingKey {
            case schemaVersion = "schema_version"
            case cases
        }
    }

    private struct GoldenCase: Decodable {
        let name: String
        let category: String
        let requests: [[String: JSONValue]]
        let assertions: [Assertion]
        let artifact: String?
    }

    private struct Assertion: Decodable {
        let pointer: String
        let equals: JSONValue
    }

    private enum JSONValue: Decodable, Equatable {
        case null, bool(Bool), number(Double), string(String), array([JSONValue]), object([String: JSONValue])

        init(from decoder: Decoder) throws {
            let value = try decoder.singleValueContainer()
            if value.decodeNil() { self = .null }
            else if let decoded = try? value.decode(Bool.self) { self = .bool(decoded) }
            else if let decoded = try? value.decode(Double.self) { self = .number(decoded) }
            else if let decoded = try? value.decode(String.self) { self = .string(decoded) }
            else if let decoded = try? value.decode([JSONValue].self) { self = .array(decoded) }
            else { self = .object(try value.decode([String: JSONValue].self)) }
        }

        var foundationValue: Any {
            switch self {
            case .null: NSNull()
            case let .bool(value): value
            case let .number(value): value
            case let .string(value): value
            case let .array(values): values.map(\.foundationValue)
            case let .object(values): values.mapValues(\.foundationValue)
            }
        }
    }

    func testSharedGoldenFlowsReplayThroughSwiftService() throws {
        let url = try XCTUnwrap(Bundle.module.url(
            forResource: "bridge-golden-flows",
            withExtension: "json",
            subdirectory: "Fixtures"
        ))
        let fixture = try JSONDecoder().decode(Fixture.self, from: Data(contentsOf: url))
        XCTAssertEqual(fixture.schemaVersion, 1)
        XCTAssertEqual(fixture.cases.count, 11)

        let root = FileManager.default.temporaryDirectory
            .appendingPathComponent("hp41-swift-golden-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: root) }

        for goldenCase in fixture.cases {
            let caseRoot = root.appendingPathComponent(goldenCase.name, isDirectory: true)
            let cardsURL = caseRoot.appendingPathComponent("cards", isDirectory: true)
            try FileManager.default.createDirectory(at: cardsURL, withIntermediateDirectories: true)
            setenv("HP41_CARDS_PATH", cardsURL.path, 1)
            defer { unsetenv("HP41_CARDS_PATH") }

            let service = try XCTUnwrap(CalculatorService(
                stateURL: caseRoot.appendingPathComponent("state.json")
            ))
            var finalResponse: Any = NSNull()
            for request in goldenCase.requests {
                let payload = request.mapValues(\.foundationValue)
                let data = try JSONSerialization.data(withJSONObject: payload, options: [.sortedKeys])
                let typedRequest = try JSONDecoder().decode(CalculatorRequest.self, from: data)
                switch service.request(typedRequest) {
                case let .success(response):
                    finalResponse = try responseObject(response)
                case let .failure(error):
                    XCTFail("\(goldenCase.name): \(error.message)")
                }
            }

            for assertion in goldenCase.assertions {
                let actual = value(at: assertion.pointer, in: finalResponse)
                XCTAssertEqual(actual, assertion.equals, "\(goldenCase.name) at \(assertion.pointer)")
            }
            if let artifact = goldenCase.artifact {
                XCTAssertTrue(FileManager.default.fileExists(
                    atPath: cardsURL.appendingPathComponent(artifact).path
                ), goldenCase.name)
            }
        }
    }

    private func responseObject(_ response: CalculatorResponse) throws -> Any {
        let stateData = try JSONEncoder().encode(response.state)
        var object = try XCTUnwrap(
            JSONSerialization.jsonObject(with: stateData) as? [String: Any]
        )
        object["status"] = response.status.rawValue
        object["error"] = response.error ?? NSNull()
        if let result = response.result {
            object["result"] = try JSONSerialization.jsonObject(with: JSONEncoder().encode(result))
        }
        return object
    }

    private func value(at pointer: String, in root: Any) -> JSONValue? {
        var current = root
        for rawComponent in pointer.split(separator: "/") {
            let component = rawComponent.replacingOccurrences(of: "~1", with: "/")
                .replacingOccurrences(of: "~0", with: "~")
            if let object = current as? [String: Any], let next = object[component] {
                current = next
            } else if let array = current as? [Any], let index = Int(component), array.indices.contains(index) {
                current = array[index]
            } else {
                return nil
            }
        }
        if current is NSNull { return .null }
        if let value = current as? Bool { return .bool(value) }
        if let value = current as? NSNumber { return .number(value.doubleValue) }
        if let value = current as? String { return .string(value) }
        return nil
    }
}
