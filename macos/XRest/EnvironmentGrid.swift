import Foundation

/// Spreadsheet view of the workspace environments: one row per variable name (the union across
/// environments, in first-seen order), one column per environment.
///
/// A cell is absent when that environment does not define the variable; absent cells are not
/// written back. Pure value type so the conversion is unit tested without any UI.
struct EnvironmentGrid {
    struct Column: Equatable {
        var name: String
        var isUnsafe: Bool
    }

    struct Row: Identifiable {
        let id = UUID()
        var name: String
        var cells: [String: NameValue]
    }

    private(set) var columns: [Column]
    private(set) var rows: [Row]

    init(environments: [EnvironmentConfig]) {
        columns = environments.map { Column(name: $0.name, isUnsafe: $0.isUnsafe) }
        var order: [String] = []
        var byName: [String: Row] = [:]
        for env in environments {
            for v in env.variables {
                if byName[v.name] == nil {
                    order.append(v.name)
                    byName[v.name] = Row(name: v.name, cells: [:])
                }
                byName[v.name]?.cells[env.name] = v
            }
        }
        rows = order.compactMap { byName[$0] }
    }

    func toConfigs() -> [EnvironmentConfig] {
        columns.map { col in
            EnvironmentConfig(
                name: col.name, isUnsafe: col.isUnsafe,
                variables: rows.compactMap { row in
                    guard var cell = row.cells[col.name] else { return nil }
                    cell.name = row.name
                    return cell
                })
        }
    }

    func isDirty(comparedTo original: [EnvironmentConfig]) -> Bool {
        toConfigs() != original
    }

    /// Problems that would make the core reject or mangle the save.
    var validationErrors: [String] {
        var errors: [String] = []
        var seen = Set<String>()
        var reported = Set<String>()
        for row in rows {
            let name = row.name.trimmingCharacters(in: .whitespaces)
            if name.isEmpty {
                if !errors.contains("A variable has no name.") { errors.append("A variable has no name.") }
            } else if !seen.insert(name).inserted, reported.insert(name).inserted {
                errors.append("Variable \(name) appears more than once.")
            }
        }
        return errors
    }

    // MARK: Editing

    mutating func addRow() {
        rows.append(Row(name: "", cells: [:]))
    }

    mutating func removeRow(id: UUID) {
        rows.removeAll { $0.id == id }
    }

    mutating func setName(rowID: UUID, _ name: String) {
        update(rowID) { $0.name = name }
    }

    mutating func setUnsafe(env: String, _ unsafe: Bool) {
        if let i = columns.firstIndex(where: { $0.name == env }) { columns[i].isUnsafe = unsafe }
    }

    mutating func setValue(rowID: UUID, env: String, _ value: String) {
        updateCell(rowID, env) { $0.value = value }
    }

    mutating func setEnabled(rowID: UUID, env: String, _ enabled: Bool) {
        updateCell(rowID, env) { $0.enabled = enabled }
    }

    /// Point the cell at a stored secret. The value is cleared; only the key is kept.
    mutating func linkSecret(rowID: UUID, env: String, key: String) {
        updateCell(rowID, env) {
            $0.secretKey = key
            $0.kind = "secret"
            $0.value = ""
        }
    }

    mutating func unlinkSecret(rowID: UUID, env: String) {
        updateCell(rowID, env) {
            $0.secretKey = nil
            $0.kind = "plain"
        }
    }

    // MARK: Helpers

    private mutating func update(_ id: UUID, _ change: (inout Row) -> Void) {
        if let i = rows.firstIndex(where: { $0.id == id }) { change(&rows[i]) }
    }

    private mutating func updateCell(_ id: UUID, _ env: String, _ change: (inout NameValue) -> Void) {
        update(id) { row in
            var cell =
                row.cells[env]
                ?? NameValue(name: row.name, value: "", enabled: true, secretKey: nil, kind: "plain")
            change(&cell)
            row.cells[env] = cell
        }
    }
}
