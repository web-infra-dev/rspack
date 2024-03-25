# Roadmap

While we do expect to support the features listed below in the future, we do not guarantee that all of the features listed below will be supported, in the next release, if at all. Additionally we do not guarantee that they will be implemented in the order listed below.

### Performance Improvement

There is still room for performance optimization and we will continue to work on this in the future with upstream projects such as [SWC](https://github.com/swc-project/swc) and [NAPI-RS](https://github.com/napi-rs/napi-rs).

### Support for more community plugins and loaders

As webpack contains a large number of APIs, we will be working to support the most frequently used loaders and plugins based on feedback from the community first.

### Module Federation support

Module Federation is a widely used webpack feature with a rich ecosystem and we plan to support module federation in the future.

### Lazy Compilation support

Although Rspack currently has good performance, there is still a lot of room to improve performance for projects with a large number of pages, and Lazy Compilation is a good way to improve performance.

### Persistent Cache Support

Persistent Cache can significantly optimize the performance of a project's warm compilation. Even though Rspack is currently able to achieve good performance on most projects, Persistent Cache can still provide a significant performance boost on some very large projects, and we plan to support this feature in the future.
