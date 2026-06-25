#if SWIFT_PACKAGE
import CHP41
#endif
import Foundation

/// Owns the opaque Rust pointers outside MainActor isolation. Normal requests
/// remain single-flight; cancellation uses a separate atomic-only handle.
final class CalculatorService: @unchecked Sendable {
    private let calculator: OpaquePointer
    private let cancellation: OpaquePointer
    private let requestLock = NSLock()

    init?(stateURL: URL) {
        guard let calculator = stateURL.path.withCString({ hp41_create($0) }) else { return nil }
        guard let cancellation = hp41_cancellation_handle(calculator) else {
            hp41_destroy(calculator)
            return nil
        }
        self.calculator = calculator
        self.cancellation = cancellation
    }

    deinit {
        hp41_cancellation_handle_destroy(cancellation)
        hp41_destroy(calculator)
    }

    func request(_ request: CalculatorRequest) -> Result<CalculatorResponse, CalculatorServiceError> {
        requestLock.lock()
        defer { requestLock.unlock() }
        do {
            let data = try JSONEncoder().encode(request)
            guard let json = String(data: data, encoding: .utf8) else {
                return .failure(.encoding("encoded request was not UTF-8"))
            }
            guard let response = json.withCString({ copyAndFree(hp41_request_json(calculator, $0)) }) else {
                return .failure(.transport)
            }
            return decode(response)
        } catch {
            return .failure(.encoding(error.localizedDescription))
        }
    }

    func state() -> Result<CalculatorResponse, CalculatorServiceError> {
        requestLock.lock()
        defer { requestLock.unlock() }
        guard let response = copyAndFree(hp41_state_json(calculator)) else {
            return .failure(.transport)
        }
        return decode(response)
    }

    func requestAsync(
        _ request: CalculatorRequest
    ) async -> Result<CalculatorResponse, CalculatorServiceError> {
        await Task.detached(priority: .userInitiated) { [self] in
            self.request(request)
        }.value
    }

    func cancel() { hp41_cancel(cancellation) }

    private func decode(_ data: Data) -> Result<CalculatorResponse, CalculatorServiceError> {
        do {
            return .success(try JSONDecoder().decode(CalculatorResponse.self, from: data))
        } catch {
            return .failure(.decoding(error.localizedDescription))
        }
    }

    private func copyAndFree(_ pointer: UnsafeMutablePointer<CChar>?) -> Data? {
        guard let pointer else { return nil }
        defer { hp41_string_free(pointer) }
        return Data(bytes: pointer, count: strlen(pointer))
    }
}
