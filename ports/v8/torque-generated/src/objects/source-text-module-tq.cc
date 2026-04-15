#include "src/objects/source-text-module.h"

namespace v8 {
namespace internal {

// Definition https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=12&c=1
class TorqueGeneratedSourceTextModuleAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(Module);
  // https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=15&c=3
  static constexpr int kCodeOffset = sizeof(Module);
  static constexpr int kCodeOffsetEnd = kCodeOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=21&c=3
  static constexpr int kRegularExportsOffset = kCodeOffsetEnd + 1;
  static constexpr int kRegularExportsOffsetEnd = kRegularExportsOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=22&c=3
  static constexpr int kRegularImportsOffset = kRegularExportsOffsetEnd + 1;
  static constexpr int kRegularImportsOffsetEnd = kRegularImportsOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=27&c=3
  static constexpr int kRequestedModulesOffset = kRegularImportsOffsetEnd + 1;
  static constexpr int kRequestedModulesOffsetEnd = kRequestedModulesOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=32&c=36
  static constexpr int kImportMetaOffset = kRequestedModulesOffsetEnd + 1;
  static constexpr int kImportMetaOffsetEnd = kImportMetaOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=37&c=3
  static constexpr int kCycleRootOffset = kImportMetaOffsetEnd + 1;
  static constexpr int kCycleRootOffsetEnd = kCycleRootOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=39&c=3
  static constexpr int kAsyncParentModulesOffset = kCycleRootOffsetEnd + 1;
  static constexpr int kAsyncParentModulesOffsetEnd = kAsyncParentModulesOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=42&c=3
  static constexpr int kDfsIndexOffset = kAsyncParentModulesOffsetEnd + 1;
  static constexpr int kDfsIndexOffsetEnd = kDfsIndexOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=43&c=3
  static constexpr int kDfsAncestorIndexOffset = kDfsIndexOffsetEnd + 1;
  static constexpr int kDfsAncestorIndexOffsetEnd = kDfsAncestorIndexOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=46&c=3
  static constexpr int kPendingAsyncDependenciesOffset = kDfsAncestorIndexOffsetEnd + 1;
  static constexpr int kPendingAsyncDependenciesOffsetEnd = kPendingAsyncDependenciesOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=48&c=3
  static constexpr int kFlagsOffset = kPendingAsyncDependenciesOffsetEnd + 1;
  static constexpr int kFlagsOffsetEnd = kFlagsOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kHeaderSize = kFlagsOffsetEnd + 1;
  static constexpr int kSize = kFlagsOffsetEnd + 1;

  static_assert(kCodeOffset == offsetof(SourceTextModule, code_),
                "Value of SourceTextModule::kCodeOffset defined in Torque and offset of field SourceTextModule::code in C++ do not match");
  static_assert(kRegularExportsOffset == offsetof(SourceTextModule, regular_exports_),
                "Value of SourceTextModule::kRegularExportsOffset defined in Torque and offset of field SourceTextModule::regular_exports in C++ do not match");
  static_assert(kRegularImportsOffset == offsetof(SourceTextModule, regular_imports_),
                "Value of SourceTextModule::kRegularImportsOffset defined in Torque and offset of field SourceTextModule::regular_imports in C++ do not match");
  static_assert(kRequestedModulesOffset == offsetof(SourceTextModule, requested_modules_),
                "Value of SourceTextModule::kRequestedModulesOffset defined in Torque and offset of field SourceTextModule::requested_modules in C++ do not match");
  static_assert(kImportMetaOffset == offsetof(SourceTextModule, import_meta_),
                "Value of SourceTextModule::kImportMetaOffset defined in Torque and offset of field SourceTextModule::import_meta in C++ do not match");
  static_assert(kCycleRootOffset == offsetof(SourceTextModule, cycle_root_),
                "Value of SourceTextModule::kCycleRootOffset defined in Torque and offset of field SourceTextModule::cycle_root in C++ do not match");
  static_assert(kAsyncParentModulesOffset == offsetof(SourceTextModule, async_parent_modules_),
                "Value of SourceTextModule::kAsyncParentModulesOffset defined in Torque and offset of field SourceTextModule::async_parent_modules in C++ do not match");
  static_assert(kDfsIndexOffset == offsetof(SourceTextModule, dfs_index_),
                "Value of SourceTextModule::kDfsIndexOffset defined in Torque and offset of field SourceTextModule::dfs_index in C++ do not match");
  static_assert(kDfsAncestorIndexOffset == offsetof(SourceTextModule, dfs_ancestor_index_),
                "Value of SourceTextModule::kDfsAncestorIndexOffset defined in Torque and offset of field SourceTextModule::dfs_ancestor_index in C++ do not match");
  static_assert(kPendingAsyncDependenciesOffset == offsetof(SourceTextModule, pending_async_dependencies_),
                "Value of SourceTextModule::kPendingAsyncDependenciesOffset defined in Torque and offset of field SourceTextModule::pending_async_dependencies in C++ do not match");
  static_assert(kFlagsOffset == offsetof(SourceTextModule, flags_),
                "Value of SourceTextModule::kFlagsOffset defined in Torque and offset of field SourceTextModule::flags in C++ do not match");
  static_assert(kSize == sizeof(SourceTextModule));
};

// Definition https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=59&c=1
class TorqueGeneratedModuleRequestAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(StructLayout);
  // https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=61&c=3
  static constexpr int kSpecifierOffset = sizeof(StructLayout);
  static constexpr int kSpecifierOffsetEnd = kSpecifierOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=65&c=3
  static constexpr int kImportAttributesOffset = kSpecifierOffsetEnd + 1;
  static constexpr int kImportAttributesOffsetEnd = kImportAttributesOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=67&c=3
  static constexpr int kFlagsOffset = kImportAttributesOffsetEnd + 1;
  static constexpr int kFlagsOffsetEnd = kFlagsOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kFlagsOffsetEnd + 1;
  static constexpr int kHeaderSize = kFlagsOffsetEnd + 1;
  static constexpr int kSize = kFlagsOffsetEnd + 1;

  static_assert(kSpecifierOffset == offsetof(ModuleRequest, specifier_),
                "Value of ModuleRequest::kSpecifierOffset defined in Torque and offset of field ModuleRequest::specifier in C++ do not match");
  static_assert(kImportAttributesOffset == offsetof(ModuleRequest, import_attributes_),
                "Value of ModuleRequest::kImportAttributesOffset defined in Torque and offset of field ModuleRequest::import_attributes in C++ do not match");
  static_assert(kFlagsOffset == offsetof(ModuleRequest, flags_),
                "Value of ModuleRequest::kFlagsOffset defined in Torque and offset of field ModuleRequest::flags in C++ do not match");
  static_assert(kSize == sizeof(ModuleRequest));
};

// Definition https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=70&c=1
class TorqueGeneratedSourceTextModuleInfoEntryAsserts {
  static constexpr int kStartOfStrongFieldsOffset = sizeof(StructLayout);
  // https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=72&c=3
  static constexpr int kExportNameOffset = sizeof(StructLayout);
  static constexpr int kExportNameOffsetEnd = kExportNameOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=73&c=3
  static constexpr int kLocalNameOffset = kExportNameOffsetEnd + 1;
  static constexpr int kLocalNameOffsetEnd = kLocalNameOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=74&c=3
  static constexpr int kImportNameOffset = kLocalNameOffsetEnd + 1;
  static constexpr int kImportNameOffsetEnd = kImportNameOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=75&c=3
  static constexpr int kModuleRequestOffset = kImportNameOffsetEnd + 1;
  static constexpr int kModuleRequestOffsetEnd = kModuleRequestOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=76&c=3
  static constexpr int kCellIndexOffset = kModuleRequestOffsetEnd + 1;
  static constexpr int kCellIndexOffsetEnd = kCellIndexOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=77&c=3
  static constexpr int kBegPosOffset = kCellIndexOffsetEnd + 1;
  static constexpr int kBegPosOffsetEnd = kBegPosOffset + kTaggedSize - 1;
  // https://crsrc.org/c/v8/src/objects/source-text-module.tq?l=78&c=3
  static constexpr int kEndPosOffset = kBegPosOffsetEnd + 1;
  static constexpr int kEndPosOffsetEnd = kEndPosOffset + kTaggedSize - 1;
  static constexpr int kEndOfStrongFieldsOffset = kEndPosOffsetEnd + 1;
  static constexpr int kStartOfWeakFieldsOffset = kEndPosOffsetEnd + 1;
  static constexpr int kEndOfWeakFieldsOffset = kEndPosOffsetEnd + 1;
  static constexpr int kHeaderSize = kEndPosOffsetEnd + 1;
  static constexpr int kSize = kEndPosOffsetEnd + 1;

  static_assert(kExportNameOffset == offsetof(SourceTextModuleInfoEntry, export_name_),
                "Value of SourceTextModuleInfoEntry::kExportNameOffset defined in Torque and offset of field SourceTextModuleInfoEntry::export_name in C++ do not match");
  static_assert(kLocalNameOffset == offsetof(SourceTextModuleInfoEntry, local_name_),
                "Value of SourceTextModuleInfoEntry::kLocalNameOffset defined in Torque and offset of field SourceTextModuleInfoEntry::local_name in C++ do not match");
  static_assert(kImportNameOffset == offsetof(SourceTextModuleInfoEntry, import_name_),
                "Value of SourceTextModuleInfoEntry::kImportNameOffset defined in Torque and offset of field SourceTextModuleInfoEntry::import_name in C++ do not match");
  static_assert(kModuleRequestOffset == offsetof(SourceTextModuleInfoEntry, module_request_),
                "Value of SourceTextModuleInfoEntry::kModuleRequestOffset defined in Torque and offset of field SourceTextModuleInfoEntry::module_request in C++ do not match");
  static_assert(kCellIndexOffset == offsetof(SourceTextModuleInfoEntry, cell_index_),
                "Value of SourceTextModuleInfoEntry::kCellIndexOffset defined in Torque and offset of field SourceTextModuleInfoEntry::cell_index in C++ do not match");
  static_assert(kBegPosOffset == offsetof(SourceTextModuleInfoEntry, beg_pos_),
                "Value of SourceTextModuleInfoEntry::kBegPosOffset defined in Torque and offset of field SourceTextModuleInfoEntry::beg_pos in C++ do not match");
  static_assert(kEndPosOffset == offsetof(SourceTextModuleInfoEntry, end_pos_),
                "Value of SourceTextModuleInfoEntry::kEndPosOffset defined in Torque and offset of field SourceTextModuleInfoEntry::end_pos in C++ do not match");
  static_assert(kSize == sizeof(SourceTextModuleInfoEntry));
};

} // namespace internal
} // namespace v8
