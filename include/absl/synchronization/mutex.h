// Minimal Abseil Mutex stub for V8 compilation on Bat_OS
// Single-threaded no-op implementation

#ifndef ABSL_SYNCHRONIZATION_MUTEX_H_
#define ABSL_SYNCHRONIZATION_MUTEX_H_

namespace absl {

class Mutex {
 public:
  Mutex() = default;
  ~Mutex() = default;

  // Non-copyable, non-movable
  Mutex(const Mutex&) = delete;
  Mutex& operator=(const Mutex&) = delete;

  void Lock() {}
  void Unlock() {}
  bool TryLock() { return true; }

  void ReaderLock() {}
  void ReaderUnlock() {}

  void WriterLock() {}
  void WriterUnlock() {}

  void AssertHeld() const {}
  void AssertReaderHeld() const {}
};

class MutexLock {
 public:
  explicit MutexLock(Mutex* mu) : mu_(mu) {
    if (mu_) mu_->Lock();
  }
  ~MutexLock() {
    if (mu_) mu_->Unlock();
  }

  MutexLock(const MutexLock&) = delete;
  MutexLock& operator=(const MutexLock&) = delete;

 private:
  Mutex* mu_;
};

class ReaderMutexLock {
 public:
  explicit ReaderMutexLock(Mutex* mu) : mu_(mu) {
    if (mu_) mu_->ReaderLock();
  }
  ~ReaderMutexLock() {
    if (mu_) mu_->ReaderUnlock();
  }

  ReaderMutexLock(const ReaderMutexLock&) = delete;
  ReaderMutexLock& operator=(const ReaderMutexLock&) = delete;

 private:
  Mutex* mu_;
};

class WriterMutexLock {
 public:
  explicit WriterMutexLock(Mutex* mu) : mu_(mu) {
    if (mu_) mu_->WriterLock();
  }
  ~WriterMutexLock() {
    if (mu_) mu_->WriterUnlock();
  }

  WriterMutexLock(const WriterMutexLock&) = delete;
  WriterMutexLock& operator=(const WriterMutexLock&) = delete;

 private:
  Mutex* mu_;
};

// Condition variable stub (V8 may reference it)
class CondVar {
 public:
  CondVar() = default;
  ~CondVar() = default;

  void Signal() {}
  void SignalAll() {}
  void Wait(Mutex*) {}
};

}  // namespace absl

#endif  // ABSL_SYNCHRONIZATION_MUTEX_H_
