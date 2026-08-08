# SPDX-License-Identifier: Apache-2.0
# SPDX-FileCopyrightText: 2019-2024 Second State INC

# WASI-Crypto plugin test target configuration

if(WASMEDGE_BUILD_TESTS AND WASMEDGE_PLUGIN_WASI_CRYPTO)
  # TODO: refactor test helper setup after upstream API update
  wasmedge_add_test(wasmedgePluginWasiCryptoTests
    wasi_crypto.cpp
  )
  target_link_libraries(wasmedgePluginWasiCryptoTests
    PRIVATE
      wasmedgePluginWasiCrypto
      gtest
      gtest_main
  )
endif()
