include_directories (${CMAKE_CURRENT_SOURCE_DIR})
include_directories(${INCLUDE_DIRS})

set( general_sources
  "cloud_label.f90"
  )

add_library(cloud_label STATIC ${general_sources})
target_link_libraries(cloud_label)


add_executable( "test_cloud_label" "test.f90" )
target_link_libraries( "test_cloud_label" cloud_label)

add_executable( "bench_cloud_label" "bench.f90" )
target_link_libraries( "bench_cloud_label" cloud_label)

install(TARGETS cloud_label DESTINATION lib)

